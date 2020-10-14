#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate gdk;
use env_logger::Env;
use gtk::Orientation::Vertical;
use gtk::{prelude::*, GtkWindowExt};
use gtk::{BaselinePosition, PackType};

use relm::{Widget, Relm};
use relm_derive::{widget, Msg};

mod config;
mod icons;
mod widgets;

use self::Msg::*;
use crate::widgets::*;

#[derive(Msg)]
pub enum Msg {
  Quit,
}

pub struct AppState {}

#[widget]
impl Widget for MainWindow {
  fn init_view(&mut self) {
    let css_str = include_bytes!("../res/gtk-light.css");
    let screen = gdk::Screen::get_default().expect("Error init gtk css provider");

    let css = gtk::CssProvider::new();
    
    if let Err(error) = css.load_from_data(css_str) {
      debug!("Failed to load theme due error {:?}", error);
    }
    gtk::StyleContext::add_provider_for_screen(
      &screen,
      &css,
      gtk::STYLE_PROVIDER_PRIORITY_USER,
    );
    gtk::StyleContext::reset_widgets(&screen);
  

    self.main_window.set_default_size(850, 300);
    self.main_window.set_resizable(false);
    self.main_panel.get_style_context().add_class("main_panel");

    //let app = self.main_window.get_application().unwrap();
    //app.send_notification(None, &gio::Notification::new("Hi"));
  }

  fn model(_relm: &Relm<Self>, _: ()) -> AppState {
    gtk::IconTheme::get_default()
      .unwrap()
      .add_resource_path("/icons");
    gtk::Settings::get_default()
      .unwrap()
      .set_property_gtk_application_prefer_dark_theme(false);

    AppState {}
  }

  fn update(&mut self, msg: Msg) {
    match msg {
      Quit => gtk::main_quit(),
    }
  }

  view! {
    #[name="main_window"]
    gtk::ApplicationWindow {
      title: "Coolio",
      icon_name: Some("tint-symbolic"),
      startup_id: "coolio_main",
      position: gtk::WindowPosition::CenterAlways,
      resizable: true,
      #[name="main_panel"]
      gtk::Box {
        orientation: Vertical,
        baseline_position: BaselinePosition::Top,
        #[name="main_notebook"]
        gtk::Notebook {
          child: {
            expand: true,
            fill: true,
            pack_type: PackType::Start
          },
          tab_pos: gtk::PositionType::Left,
          show_border: false,
          #[name="cooling_page"]
          CoolingPage {
            child: {
              tab_expand: false,
              tab_fill: true,
              tab_label: Some("Cooling")
            },
          }
        },
      },
      delete_event(_, _) => (Quit, Inhibit(false)),
    }
  }
}

fn main() {
  env_logger::from_env(Env::default().default_filter_or("debug")).init();

  let res_bytes = include_bytes!("icons.bin");
  let data = glib::Bytes::from(&res_bytes[..]);
  let resource = gio::Resource::from_data(&data).unwrap();
  gio::resources_register(&resource);

  MainWindow::run(()).expect("MainWindow::run");
}
