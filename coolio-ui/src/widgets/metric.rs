use coolio_drivers::Metric;
use gtk::prelude::*;
use gtk::{Orientation, StyleContextExt};
use relm::{Relm, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg, Clone)]
pub enum MetricMsg {
  Update(Metric),
}

use self::MetricMsg::*;

#[derive(Clone)]
pub struct Model {
  visible: bool,
  value: f64,
  unit: String,
  label: String,
  critical_value: f64,
  human_value: String,
}

fn capitalize(s: &str) -> String {
  let mut c = s.chars();
  match c.next() {
    None => String::new(),
    Some(f) => f.to_uppercase().chain(c).collect(),
  }
}

fn humanize(value: String) -> String {
  value
    .split(".")
    .skip(1)
    .take(2)
    .map(|s| match s {
      "cpu" => "CPU".to_string(),
      other => capitalize(other),
    })
    .collect::<Vec<_>>()
    .join(" ")
}

#[widget]
impl Widget for MetricWidget {
  fn init_view(&mut self) {
    self.metric.get_style_context().add_class("metric-panel");
    self.metric.get_style_context().add_class("normal");
    self.metric_value.get_style_context().add_class("value");
    self.metric_unit.get_style_context().add_class("unit");
    self.metric_label.get_style_context().add_class("label");
    self.container.get_style_context().add_class("metric");
  }

  fn model(_relm: &Relm<Self>, params: Metric) -> Model {
    let label: String = humanize(params.name().to_string());
    let unit = params.unit().to_string();
    Model {
      visible: true,
      value: params.value(),
      unit,
      label,
      critical_value: params.max_value(),
      human_value: params.human_value(),
    }
  }

  fn update(&mut self, event: MetricMsg) {
    match event {
      Update(metric) => {
        self.model.value = metric.value();
        self.model.human_value = metric.human_value();
        let burden = self.model.value / self.model.critical_value;
        // remove any status class
        let ctx = self.metric.get_style_context();
        if burden >= 0.9 {
          ctx.remove_class("warn");
          ctx.remove_class("normal");
          ctx.add_class("critical");
        } else if burden >= 0.7 {
          ctx.remove_class("critical");
          ctx.remove_class("normal");
          ctx.add_class("warn");
        } else {
          ctx.remove_class("warn");
          ctx.remove_class("critical");
          ctx.add_class("normal");
        }
      }
    }
  }

  view! {
    #[name="container"]
    gtk::FlowBoxChild {
      visible: self.model.visible,
      can_focus: false,
      #[name="metric"]
      gtk::Box {
        orientation: Orientation::Vertical,
        can_focus: false,
        visible: true,
        homogeneous: false,
        gtk::Box {
          orientation: Orientation::Horizontal,
          can_focus: false,
          // draw(wdg, cr) => (Paint(wdg.clone(), cr.clone()), Inhibit(false)),
          child: { fill: true, expand: true, },
          #[name="metric_value"]
          gtk::Label {
            child: { fill: true, expand: true },
            halign: gtk::Align::End,
            valign: gtk::Align::Baseline,
            text: &self.model.human_value
          },
          #[name="metric_unit"]
          gtk::Label {
            child: { fill: true, expand: true},
            halign: gtk::Align::Start,
            valign: gtk::Align::Baseline,
            text: &self.model.unit.to_string()
          },
        },
        #[name="metric_label"]
        gtk::Label {
          child: {
            fill: true,
            expand: false,
          },
          text: &self.model.label
        }
      }
    }
  }
}
