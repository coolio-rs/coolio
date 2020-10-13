use sysinfo::RefreshKind;
use crate::DeviceManager;
use std::sync::Mutex;
use std::sync::Arc;
use sysinfo::{ComponentExt, ProcessorExt, SystemExt, System};
use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const UNIT_TEMPERATURE: &'static str = "°C";
const UNIT_DUTY: &'static str = "%";


#[derive(Clone, Debug)]
pub struct Metric(String, f64, String, f64);

impl Metric {
  pub fn new<T: Into<String>>(name: T, value: f64, unit: T, max_value: f64) -> Self {
    Metric(name.into(), value, unit.into(), max_value)
  }

  pub fn new_temperature(name: &str, value: f64, max_value: f64) -> Self {
    Self::new(name, value, UNIT_TEMPERATURE, max_value)
  }

  pub fn new_duty(name: &str, value: f64, max_value: f64) -> Self {
    Self::new(name, value, UNIT_DUTY, max_value)
  }

  pub fn path(&self) -> Vec<&str> {
    self.0.split(".").collect::<Vec<_>>()
  }

  pub fn name(&self) -> &str {
    &self.0
  }

  pub fn value<T: From<f64>>(&self) -> T {
    self.1.into()
  }

  pub fn unit(&self) -> &str {
    &self.2
  }

  pub fn max_value<T: From<f64>>(&self) -> T {
    self.3.into()
  }

  pub fn is(&self, metric_name: &str) -> bool {
    self.0 == metric_name
  }

  pub fn human_value(&self) -> String {
    let value: f64 = self.1;
    match self.2.as_str() {
      "%" => format!("{:.0}", value),
      "rpm" => format!("{:.0}", value),
      "°C" => format!("{:.1}", value),
      _ => format!("{:.2}", value),
    }
  }
}

fn fetch_metrics(system: &mut System) -> Vec<Metric> {
  let mut metrics: Vec<Metric> = vec![];
  system.refresh_cpu();
  system.refresh_components();
  let processor = system.get_global_processor_info();
  metrics.push(Metric::new("dev.cpu.user", processor.get_cpu_usage() as f64, UNIT_DUTY, 100.0));
  for component in system.get_components() {
    match component.get_label() {
      "Package id 0" => {
        let metric = Metric::new(
          "dev.cpu.heat",
          component.get_temperature() as f64,
          UNIT_TEMPERATURE,
          component.get_critical().map(|v| v as f64).unwrap()
        );
        metrics.push(metric)
      },
      _ => {}
    }
  }
  metrics
}

pub struct MetricCollector {
  stop_tx: Mutex<Sender<()>>,
  metrics: Arc<Mutex<Vec<Metric>>>
}

impl Drop for MetricCollector {
  fn drop(&mut self) {
    self.stop_tx.lock().unwrap().send(()).unwrap_or(());
  }
}

impl MetricCollector {
  pub fn new() -> Self {
    let (stop_tx, stop_rx): (Sender<()>, Receiver<()>) = mpsc::channel();
    let metrics: Arc<Mutex<Vec<Metric>>> = Arc::new(Mutex::new(vec![]));
    let counters = Arc::clone(&metrics);
    thread::spawn(move || {
      let refresh_kind = RefreshKind::new()
        .with_cpu()
        .with_components()
        .with_components_list();
      let system = &mut System::new_with_specifics(refresh_kind);
      let device_manager = &mut DeviceManager::new().unwrap();
      let pause = 3000;
      loop {
        match stop_rx.try_recv() {
          Ok(_) | Err(TryRecvError::Disconnected) => break,
          Err(TryRecvError::Empty) => {}
        }
        let results = &mut fetch_metrics(system);
        let device_status_result = device_manager.device_status();
        {
          let mut countes_inner = counters.lock().unwrap();
          countes_inner.clear();
          while let Some(metric) = results.pop() {
            countes_inner.push(metric);
          }
          if let Ok(device_status) = device_status_result {
            let mut metrics: Vec<Metric> = device_status.counters; 
            while let Some(counter) = metrics.pop() {
              countes_inner.push(counter);
            }
          }
        }
        thread::sleep(Duration::from_millis(pause));
      }
    });
    Self {
      stop_tx: Mutex::new(stop_tx),
      metrics
    }
  }

  pub fn read_last(&self) -> Vec<Metric> { 
    let metrics = self.metrics.lock().unwrap().clone();
    // metrics.reverse();
    metrics
  }
}


#[cfg(test)]
mod metrics_test {
  use crate::metrics::MetricCollector;
  use std::thread;
  use std::time::Duration;
  
  #[test]
  fn should_read_cpu_temperature() {
    let collector = MetricCollector::new();
    thread::sleep(Duration::from_millis(1000));
    let result = collector.read_last();
    assert_eq!(result[1].name(), "dev.cpu.user");
    assert_eq!(result[0].name(), "dev.cpu.heat");
  }
}