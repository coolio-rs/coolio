// use coolio_drivers::CoolingConf;
use coolio_drivers::{CoolingConf, Metric, MonitorHeat};
use gdk::RGBA;
use gtk::prelude::*;
use gtk::{ImageExt, StyleContextExt, WidgetExt};
use relm::Relm;
use relm_derive::{widget, Msg};
use std::ops::Rem;
use std::str::FromStr;

// const PIN_ICON: &str = "find-location-symbolic";
use self::ProfileConfigMsg::*;

#[derive(Msg, Clone)]
pub enum ProfileConfigMsg {
  OnPanelButtonDown(gdk::EventButton),
  OnPanelButtonUp(gdk::EventButton),
  OnPanelMouseMove(gdk::EventMotion),
  DrawLineChart(gtk::Image, cairo::Context),
  UpdateMetric(Metric),
  SelectSensor(Option<String>),
  SetProfile(CoolingConf),
  Ignore,
}

const CHART_TEXT_SPACE_WIDTH: f64 = 50.0;
const CHART_TEXT_SPACE_HEIGHT: f64 = 64.0;

lazy_static! {
  static ref CLR_BLU: RGBA = RGBA::from_str("#3a6e96").expect("error while parsing color");
  static ref CLR_RED: RGBA = RGBA::from_str("#e9724f").expect("error while parsing color");
  static ref CLR_YLW: RGBA = RGBA::from_str("#e9c64f").expect("error while parsing color");
  static ref CLR_GRAY: RGBA = RGBA::from_str("gray").expect("error while parsing color");
  static ref LIQUID_COLOR: RGBA = RGBA::from_str("#80ced6").expect("color parsing eror");
  static ref CPU_COLOR: RGBA = RGBA::from_str("#d64161").expect("color parsing eror");
  static ref DISABLED_BG_COLOR: RGBA = RGBA::from_str("#a2b9bc").expect("color parsing eror");
}

#[derive(Clone)]
pub struct Model {
  relm: Relm<ProfileConfig>,
  profile: Vec<(u8, u8)>,
  device: String,
  liquid_temp: f64,
  cpu_temp: f64,
  selected_sensor: Option<String>,
  height: i32,
  width: i32,
  selected_index: Option<usize>,
}

#[widget]
impl relm::Widget for ProfileConfig {
  fn init_view(&mut self) {
    self.monitor_sensor.append(Some("cpu"), "CPU Temperature");
    self
      .monitor_sensor
      .append(Some("liquid"), "Liquid Temperature");
    self
      .monitor_sensor
      .set_active_id(self.model.selected_sensor.as_ref().map(|v| v.as_str()));

    let style = self.chart.get_style_context();
    style.add_class("p-0");
    style.add_class("m-0");
    style.add_class("chart");

    self.event_box.get_style_context().add_class("p-0");
    self.event_box.get_style_context().add_class("border-1");

    self.chart.set_valign(gtk::Align::Start);
    self.chart.set_halign(gtk::Align::Center);

    self
      .box1
      .get_style_context()
      .add_class("cooling-dev-config");
    self.box1.get_style_context().add_class("m-0");
    self.box1.get_style_context().add_class("p-0");

    self.draw_chart();
  }

  fn model(relm: &Relm<Self>, (device,): (String,)) -> Model {
    Model {
      relm: relm.clone(),
      device,
      profile: (0..16).map(|s| (s * 5 + 20, 50)).collect::<Vec<_>>(),
      liquid_temp: 0.0,
      cpu_temp: 0.0,
      selected_sensor: Some("cpu".to_string()),
      width: 600,
      height: 300,
      selected_index: None,
    }
  }

  fn update(&mut self, event: ProfileConfigMsg) {
    match event {
      SetProfile(profile) => match profile {
        CoolingConf::VariableSpeed(sensor, p) => {
          self.model.selected_sensor = Some(sensor.to_string());
          self.model.profile = p;
        }
        CoolingConf::FixedSpeed(p) => {
          self.model.selected_sensor = Some(MonitorHeat::Liquid.to_string());
          self.model.profile = (0..16).map(|s| (s * 5 + 20, p)).collect::<Vec<_>>();
        }
      },
      OnPanelButtonDown(button) => {
        let pos = button.get_position();
        let (x, y) = self.point_to_profile(pos);
        self.model.selected_index =
          self.model.profile.iter().position(|&p| {
            (p.0 as i32 - x as i32).abs() <= 1 && (p.1 as i32 - y as i32).abs() <= 2
          });
      }
      OnPanelButtonUp(button) => {
        let point = button.get_position();
        let (temp, duty) = self.point_to_profile(point);
        debug!("{} {}", temp, duty);
        match button.get_button() {
          1 => {
            // felt mouse button
            self.model.selected_index = None
          }
          3 => {
            // right mouse button
          }
          _ => (),
        }
      }
      OnPanelMouseMove(motion) => {
        let point = motion.get_position();
        let (_temp, duty) = self.point_to_profile(point);
        if let Some(index) = self.model.selected_index {
          let temp = self.model.profile[index].0;
          let duty = self.duty_min_max(duty);
          for p in self.model.profile.iter_mut() {
            if p.0 < temp {
              p.1 = duty.min(p.1)
            } else if p.0 == temp {
              p.1 = duty
            } else {
              p.1 = duty.max(p.1)
            }
          }
        }
      }
      DrawLineChart(_img, _cr) => {
        self.draw_chart();
      }
      UpdateMetric(metric) => match metric.path().as_slice() {
        ["dev", "cpu", "heat"] => self.model.cpu_temp = metric.value(),
        ["dev", "krakenX", "liquid"] => self.model.liquid_temp = metric.value(),
        ["dev", "krakenM", "liquid"] => self.model.liquid_temp = metric.value(),
        _ => (),
      },
      SelectSensor(val) => self.model.selected_sensor = val,
      Ignore => (),
    }
  }

  fn draw_chart(&mut self) {
    // debug!("{:?}", sprite);
    let surface =
      cairo::ImageSurface::create(cairo::Format::ARgb32, self.model.width, self.model.height)
        .unwrap();
    let cr = cairo::Context::new(&surface);

    cr.set_font_size(12.0);

    for step in 0..=4 {
      cr.set_source_rgba(0.9, 0.9, 0.9, 0.7);
      if step.rem(4) == 0 {
        cr.set_dash(&[], 0.0);
      } else {
        cr.set_dash(&[3.0, 3.0], 0.0);
      }

      let duty = step * 25;
      let (x, y) = self.profile_to_point((0, duty));
      cr.move_to(x, y);
      let (x, y) = self.profile_to_point((100, duty));
      cr.line_to(x, y);
      cr.stroke();

      let text = format!("{}%", duty);
      let ext = cr.text_extents(&text);
      cr.move_to(
        CHART_TEXT_SPACE_WIDTH - ext.width - 5.0,
        y + ext.height / 2.0,
      );
      cr.set_source_rgba(0.6, 0.6, 0.6, 0.9);
      cr.show_text(&text);
    }

    for step in 1..=10 {
      let temp = step * 10;
      let (x, y) = self.profile_to_point((temp, 0));

      let text = format!("{}°C", temp);
      let ext = cr.text_extents(&text);
      cr.move_to(x - ext.width / 2.0, y + ext.height + 10.0);
      cr.set_source_rgba(0.6, 0.6, 0.6, 0.9);
      cr.show_text(&text);
    }

    // CPU / Liquid temperature line and label
    let current_temp = if self.model.selected_sensor == Some("cpu".to_string()) {
      cr.set_source_rgba(CPU_COLOR.red, CPU_COLOR.green, CPU_COLOR.blue, 1.0);
      self.model.cpu_temp
    } else {
      cr.set_source_rgba(LIQUID_COLOR.red, LIQUID_COLOR.green, LIQUID_COLOR.blue, 1.0);
      self.model.liquid_temp
    };
    cr.set_dash(&[2.0, 2.0], -1.0);
    let (x1, y1) = self.profile_to_point((current_temp, 0.0));
    let (x2, y2) = self.profile_to_point((current_temp, 100.0));
    cr.move_to(x1, y1);
    cr.line_to(x2, y2 - 10.0);
    cr.stroke();
    cr.set_dash(&[], 0.0);
    let text = format!("{:.0}°C", current_temp);
    let ext = cr.text_extents(&text);
    cr.rectangle(
      x2 - ext.width / 2.0 - 3.0,
      y2 - 24.0 - 3.0,
      ext.width + 6.0,
      ext.height + 6.0,
    );
    cr.fill();
    cr.move_to(x2 - ext.width / 2.0, y2 + ext.height - 24.0);
    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
    cr.show_text(&text);

    // PROFILE interactive line chart
    cr.set_source_rgb(CLR_BLU.red, CLR_BLU.green, CLR_BLU.blue);
    let (mut start_x, mut start_y) = self.profile_to_point(self.model.profile[0]);
    cr.arc(start_x, start_y, 6.0, 0.0, 2.0 * std::f64::consts::PI);
    cr.fill();
    for &profile in self.model.profile.as_slice().iter().skip(1) {
      cr.move_to(start_x, start_y);
      let (xc, yc) = self.profile_to_point(profile);
      cr.line_to(xc, yc);
      start_x = xc;
      start_y = yc;
      cr.stroke();
      cr.arc(xc, yc, 6.0, 0.0, 2.0 * std::f64::consts::PI);
      cr.fill();
    }

    //cr.rectangle(0.0, 0.0, self.model.width.into(), self.model.height.into());
    cr.stroke();
    self.chart.set_from_surface(Some(&surface));
  }

  fn point_to_profile(&self, (x, y): (f64, f64)) -> (u8, u8) {
    let mut translate = self.get_matrix();
    translate.invert();
    let (a, b) = translate.transform_point(x, y);
    (a as u8, b as u8)
  }

  fn profile_to_point<T: Into<f64>>(&self, (temp, duty): (T, T)) -> (f64, f64) {
    self.get_matrix().transform_point(temp.into(), duty.into())
  }

  fn get_matrix(&self) -> cairo::Matrix {
    let width = self.model.width as f64;
    let height = self.model.height as f64;

    let xx = (width - CHART_TEXT_SPACE_WIDTH * 2.0) / 100.0;
    let yy = (height - CHART_TEXT_SPACE_HEIGHT) / 100.0;
    cairo::Matrix::new(
      xx,
      0.0,
      0.0,
      -yy,
      CHART_TEXT_SPACE_WIDTH,
      height - CHART_TEXT_SPACE_HEIGHT * 0.5,
    )
  }

  fn duty_min_max(&mut self, duty: u8) -> u8 {
    if self.model.device == "pump".to_string() {
      duty.min(100).max(50)
    } else {
      duty.min(100).max(25)
    }
  }

  view! {
    #[name="box1"]
    gtk::Box {
      orientation: gtk::Orientation::Vertical,
      homogeneous: false,
      spacing: 10,
      gtk::Toolbar {
        orientation: gtk::Orientation::Horizontal,
        child: {
          expand: false,
          fill: true
        },
        gtk::ToolItem {
          item: {expand: true},
          gtk::Label {
            text: &self.model.device.to_uppercase(),
            halign: gtk::Align::Start
          }
        },
        gtk::ToolItem {
          item: { homogeneous: false, expand: false },
          gtk::Label { text: "Configure for:" },
        },
        gtk::ToolItem {
          item: { homogeneous: false, expand: false },
          #[name="monitor_sensor"]
          gtk::ComboBoxText {
            changed(val) => SelectSensor(val.get_active_id().map(|s| s.to_string()))
          },
        }
      },
      #[name="event_box"]
      gtk::EventBox {
        child: {
          expand: false,
          fill: false
        },
        halign: gtk::Align::Center,
        valign: gtk::Align::Center,
        button_press_event(_, button) => (OnPanelButtonDown(button.clone()), Inhibit(false)),
        button_release_event(_, button) => (OnPanelButtonUp(button.clone()), Inhibit(false)),
        motion_notify_event(_, motion) => (OnPanelMouseMove(motion.clone()), Inhibit(false)),
        #[name="chart"]
        gtk::Image {
          valign: gtk::Align::Center,
          halign: gtk::Align::Center,
          draw(w, cr) => (DrawLineChart(w.clone(), cr.clone()), Inhibit(false)),
        },
      }
    },
  }
}
