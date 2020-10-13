#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate hidapi;

// use std::sync::mpsc::{Sender, Receiver, TryRecvError};
// use std::sync::mpsc;
// use std::thread;
// use std::time::Duration;

/// Holds types and structures required to configure supported devices.
mod config;
/// Arbitary driver implmentation
mod driver;
/// Supported drivers
mod drivers;
/// Manages discovered device
mod device_manager;
mod metrics;

pub(crate) mod utils;

pub use crate::device_manager::DeviceManager;
pub use crate::config::{
  ColorMode, DeviceConf, CoolingProfile, MonitorHeat, Speed, SubsystemChannel, Temperature,
};
pub use driver::{Driver, DriverError, DeviceStatus};
pub use self::metrics::*;

// pub fn init() -> {
//   let (stop_tx, stop_rx): (Sender<()>, Receiver<()>) = mpsc::channel();
//   thread::spawn(move || {
//     loop {

//     }  
//   })
// }