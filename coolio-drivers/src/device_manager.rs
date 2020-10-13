use crate::{DeviceStatus, Driver, DeviceConf, DriverError};
use crate::drivers::{KrakenGen3, KrakenModelSeries};
// use std::collections::VecDeque;

lazy_static! {
  static ref HAPI: hidapi::HidApi = {
    hidapi::HidApi::new().unwrap()
  };
}

pub struct DeviceManager {
  driver: Box<dyn Driver>,
  // command_queue: VecDeque
}

impl DeviceManager {
  pub fn new() -> Result<Self, DriverError> {
    let api = hidapi::HidApi::new()?;
    
    if let Some(driver) = resolve_driver(&api) {
      Ok(Self { driver })
    } else {
      Err(DriverError::NoDeviceFound)
    }
  }

  fn write(&self, channel: &str, cfg: DeviceConf) -> Result<(), DriverError> {
    let device = self.driver.device_info().open_device(&HAPI)?;
    let writes = self.driver.encode(channel, cfg)?;
    let mut write_result: Result<(), DriverError> = Ok(());

    for buf in writes.iter() {
      write_result = match device.write(buf.as_slice()) {
        Err(e) => Err(e.into()),
        Ok(_) => Ok(()),
      };
      if write_result.is_err() {
        break;
      }
    }

    write_result
  }

  pub fn device_status(&mut self) -> Result<DeviceStatus, DriverError> {
    let mut vec = (1..=self.driver.read_size())
      .map(|_| 0u8)
      .collect::<Vec<_>>();
    let buf: &mut [u8] = vec.as_mut_slice();
    let device = self.driver.device_info().open_device(&HAPI)?;
    let read_length = device.read(buf)?;
    self.driver.read_status(buf, read_length)
  }

  // fn feature_report(&self) -> Result<(), DriverError> {
  //   let mut vec = (1..=self.driver.read_size())
  //     .map(|_| 0u8)
  //     .collect::<Vec<_>>();

  //   vec[0] = 0x81u8;
  //   let buf: &mut [u8] = vec.as_mut_slice();
  //   let device = self.driver.device_info().open_device(&self.api)?;
  //   println!("{:?}", device.get_feature_report(buf)?);
  //   Ok(())
  // }
}

fn resolve_driver(api: &hidapi::HidApi) -> Option<Box<dyn Driver>> {
  api
    .device_list()
    .find_map(|dev| match (dev.vendor_id(), dev.product_id()) {
      (0x1e71, 0x170e) => Some(KrakenGen3::new(
        dev.clone(),
        "NZXT Kraken X (X42, X52, X62 or X72)",
        KrakenModelSeries::KrakenX,
      )),
      (0x1e71, 0x1715) => Some(KrakenGen3::new(
        dev.clone(),
        "NZXT Kraken M22",
        KrakenModelSeries::KrakenM,
      )),
      _ => None,
    })
}
