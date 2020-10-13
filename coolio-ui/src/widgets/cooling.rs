use crate::config::AppConfig;
use crate::icons::Icon;
use crate::widgets::{MetricMsg, MetricWidget, ProfileConfig, ProfileConfigMsg};
use coolio_drivers::{Metric, MetricCollector, DeviceManager};
use gtk::prelude::*;
use gtk::{BaselinePosition, FlowBoxExt, Orientation, StyleContextExt, WidgetExt};
use relm::interval;
use relm::{Component, ContainerWidget, Relm, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum CoolingMsg {
  Save,
  SaveAs,
  Delete,
  SelectMasterProfile,
  UpdateMetric(Metric),
  AddMetric(Metric),
  UpdateDeviceStatus,
  Ignore
}

use CoolingMsg::*;


pub struct Cooling {
  relm: Relm<CoolingPage>,
  config: AppConfig,
  counters: Vec<(String, Component<MetricWidget>)>,
  device_manager: DeviceManager,
  metric_collector: MetricCollector
}

fn monitored() -> Vec<Metric> {
  vec![
    Metric::new("dev.cpu.heat", 0.0, "°C", 100.0),
    Metric::new("dev.cpu.user", 0.0, "%", 100.0),
    Metric::new("dev.krakenX.liquid", 0.0, "°C", 60.0),
    Metric::new("dev.krakenX.fan", 0.0, "rpm", 1800.0),
    Metric::new("dev.krakenX.pump", 0.0, "rpm", 2700.0),
  ]
}

#[widget]
impl Widget for CoolingPage {
  fn init_view(&mut self) {
    for p in self.model.config.profiles.as_slice() {
      self.master_profile_combobox.append(Some(&p.name), &p.name);
    }
    self.master_profile_combobox.set_active_id(
      self
        .model
        .config
        .selected_profile
        .as_ref()
        .map(String::as_str),
    );
    self.cooling_box.get_style_context().add_class("p-10");
    self
      .main_profile_label
      .get_style_context()
      .add_class("pr-5");
    let mut metrics = monitored();
    metrics.reverse();
    while let Some(metric) = metrics.pop() {
      self.model.relm.stream().emit(AddMetric(metric.clone()));
    }
  }

  fn model(relm: &Relm<Self>, _params: ()) -> Cooling {
    let config = AppConfig::load();
    let device_manager = DeviceManager::new().unwrap();
    let metric_collector = MetricCollector::new();
    Cooling {
      relm: relm.clone(),
      config,
      counters: vec![],
      device_manager,
      metric_collector
    }
  }

  fn subscriptions(&mut self, relm: &Relm<Self>) {
    interval(relm.stream(), 2000, || UpdateDeviceStatus);
  }

  fn update(&mut self, msg: CoolingMsg) {
    match msg {
      UpdateDeviceStatus => {
        for metric in self.model.metric_collector.read_last() {
          self.model.relm.stream().emit(UpdateMetric(metric.clone()));
          self
            .fan_profile
            .emit(ProfileConfigMsg::UpdateMetric(metric.clone()));
          self
            .pump_profile
            .emit(ProfileConfigMsg::UpdateMetric(metric.clone()));
        }
      }
      SelectMasterProfile => {
        let val = self.master_profile_combobox.get_active_id();

        debug!("changing master profile to {:?}", val);
        self.model.config.selected_profile = if let Some(val) = val {
          Some(val.to_string())
        } else {
          None
        };
        let new_current = self.model.config.current();
        self
          .fan_profile
          .emit(ProfileConfigMsg::SetProfile(new_current.fan));
        self
          .pump_profile
          .emit(ProfileConfigMsg::SetProfile(new_current.pump));
      }
      AddMetric(metric) => {
        let key = metric.name();
        let widget = self
          .counter_widgets
          .add_widget::<MetricWidget>(metric.clone());
        self.model.counters.push((key.into(), widget));
      }
      UpdateMetric(metric) => {
        self
          .fan_profile
          .emit(ProfileConfigMsg::UpdateMetric(metric.clone()));
        self
          .pump_profile
          .emit(ProfileConfigMsg::UpdateMetric(metric.clone()));
        if let Some((_, widget)) = self.model.counters.iter().find(|(k, _)| metric.is(k)) {
          widget.emit(MetricMsg::Update(metric));
        }
      }
      _ => (),
    }
  }

  view! {
    #[name="cooling_box"]
    gtk::Box {
      orientation: Orientation::Vertical,
      baseline_position: BaselinePosition::Top,
      spacing: 10,
      #[name="cooling_toolbar"]
      gtk::Toolbar {
        child: {
          fill: true,
          expand: false,
        },
        gtk::ToolItem {
          item: {
            homogeneous: false,
            expand: false
          },
          #[name="main_profile_label"]
          gtk::Label {
            label: "Choose a master profile:"
          },
        },
        gtk::ToolItem {
          item: {
            homogeneous: false,
            expand: false,
          },
          #[name="master_profile_combobox"]
          gtk::ComboBoxText {
            can_focus: true,
            changed(_) => SelectMasterProfile
          }
        },
        gtk::SeparatorToolItem {
          item: {
            expand: true
          }
        },
        #[name="save_profile"]
        gtk::ToolButton {
          item: {
            homogeneous: false,
            expand: false,
          },
          label: Some("Save"),
          icon_name: Icon::SAVE.to_icon_name()
        },
        #[name="save_profile_as"]
        gtk::ToolButton {
          item: {
            homogeneous: false,
            expand: false,
          },
          label: Some("Save as"),
          icon_name: Icon::COPY.to_icon_name()
        },
        #[name="delete_profile"]
        gtk::ToolButton {
          item: {
            homogeneous: false,
            expand: false,
          },
          label: Some("Delete as"),
          icon_name: Icon::TRASH.to_icon_name()
        },
      },
      #[name="counter_widgets"]
      gtk::FlowBox {
        child: {
          fill: true,
          expand: false
        },
        homogeneous: true,
        min_children_per_line: 5,
        max_children_per_line: 5,
        selection_mode: gtk::SelectionMode::None,
        activate_on_single_click: false,
      },
      gtk::Box {
        baseline_position: BaselinePosition::Top,
        orientation: Orientation::Vertical,
        homogeneous: true,
        spacing: 10,
        child: {
          fill: true,
          expand: false
        },
        #[name="fan_profile"]
        ProfileConfig(("fan".to_string(), Icon::WIND)) {
          child: {
            fill: true,
            expand: true,
          },
        },
        #[name="pump_profile"]
        ProfileConfig(("pump".to_string(), Icon::TINT)) {
          child: {
            fill: true,
            expand: true,
          },
        }
      }
    }
  }
}

impl From<CoolingMsg> for ProfileConfigMsg {
  fn from(msg: CoolingMsg) -> Self {
    match msg {
      CoolingMsg::UpdateMetric(metric) => ProfileConfigMsg::UpdateMetric(metric),
      _ => ProfileConfigMsg::Ignore,
    }
  }
}
