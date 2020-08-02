#[macro_use]
extern crate bitflags;
/// Holds types and structures required to configure supported devices.
mod config;
mod driver;
/// Supported drivers
mod drivers;
pub mod utils;

pub use crate::config::{
  ColorMode, CoolingConf, CoolingProfile, MonitorHeat, Speed, SubsystemChannel, Temperature,
};
pub use driver::{Driver, DriverError};

#[derive(Clone)]
pub struct DeviceStatus {
  pub description: String,
  pub liquid: Option<f32>,
  pub fan: Option<f32>,
  pub pump: Option<f32>,
  pub firmware: Option<(u16, u16, u16)>,
}

#[derive(Clone, Debug)]
pub struct Metric(String, f64, String, f64);

impl Metric {
  pub fn new<T: Into<String>>(name: T, value: f64, unit: T, max_value: f64) -> Self {
    Metric(name.into(), value, unit.into(), max_value)
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
      "%" => format!("{:.0}", 100.0 * value),
      "rpm" => format!("{:.0}", value),
      "°C" => format!("{:.1}", value),
      _ => format!("{:.2}", value),
    }
  }
}
