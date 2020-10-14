use crate::utils::{interpolate_profile, normalize_profile};
use crate::{Driver, DriverError, DeviceStatus};
use crate::config::DeviceConf;
use crate::metrics::Metric;
use hidapi::DeviceInfo;

const CRITICAL_TEMPERATURE: u8 = 60;
const MIN_DUTY: u8 = 60;
const READ_LENGTH: usize = 64;

#[derive(PartialEq, Copy, Clone)]
pub enum KrakenModelSeries {
  KrakenX,
  KrakenM,
}

pub struct KrakenGen3 {
  device_info: DeviceInfo,
  firmware_version: Option<(u16, u16, u16)>,
  description: String,
  kraken_model_series: KrakenModelSeries,
  supports_cooling: bool,
}

impl KrakenGen3 {
  pub fn new<T: Into<String>>(
    device_info: DeviceInfo,
    description: T,
    model_series: KrakenModelSeries,
  ) -> Box<dyn Driver> {
    let supports_cooling = model_series != KrakenModelSeries::KrakenM;
    Box::new(Self {
      device_info,
      firmware_version: None,
      description: description.into(),
      kraken_model_series: model_series,
      supports_cooling,
    })
  }

  fn get_speed_channel_config(&self, channel: &str) -> Result<(u8, u8, u8), DriverError> {
    match channel {
      "fan" => Ok((0x80, 25, 100)),
      "pump" => Ok((0xc0, 50, 100)),
      _ => Err(DriverError::NotSupported(format!(
        "Speed channel {} is not supported by {} device",
        channel, self.description
      ))),
    }
  }

  fn encode_speed_profile(
    &self,
    channel: &str,
    profile: Vec<(u8, u8)>,
  ) -> Result<Vec<Vec<u8>>, DriverError> {
    let (ch_base, duty_min, duty_max) = self.get_speed_channel_config(channel)?;
    let stdtemps = (20u8..50u8)
      .chain((50u8..=60u8).step_by(2))
      .collect::<Vec<_>>();
    let normalized = normalize_profile(profile, CRITICAL_TEMPERATURE, duty_min);
    let interpolated_profile = stdtemps
      .iter()
      .enumerate()
      .map(|(i, &temperature)| {
        let duty = interpolate_profile(&normalized, temperature, CRITICAL_TEMPERATURE)
          .min(duty_max)
          .max(duty_min);
        vec![0x2, 0x4d, ch_base + (i as u8), temperature, duty]
      })
      .collect::<Vec<_>>();

    Ok(interpolated_profile)
  }

  fn encode_fixed_speed(
    &self,
    channel: &'static str,
    duty: u8,
  ) -> Result<Vec<Vec<u8>>, DriverError> {
    if self.supports_cooling {
      self.encode_speed_profile(channel, vec![(0, duty), (59, duty), (60, 100), (100, 100)])
    } else {
      self.encode_instantaneous_speed(channel, duty)
    }
  }

  fn encode_instantaneous_speed(
    &self,
    channel: &str,
    duty: u8,
  ) -> Result<Vec<Vec<u8>>, DriverError> {
    let (ch_base, duty_min, duty_max) = self.get_speed_channel_config(channel)?;
    let duty = duty.max(duty_min).min(duty_max);
    debug!("setting {} duty to {}%", channel, duty);
    let result = vec![vec![0x2, 0x4d, ch_base & 0x70, 0, duty]];
    Ok(result)
  }
}

impl Driver for KrakenGen3 {
  // type Args = ();

  fn description(&self) -> String {
    self.description.clone()
  }

  fn vendor_id(&self) -> u16 {
    self.device_info.vendor_id()
  }

  fn product_id(&self) -> u16 {
    self.device_info.vendor_id()
  }

  fn release_number(&self) -> Option<String> {
    if let Some((a, b, c)) = self.firmware_version {
      Some(format!("{}.{}.{}", a, b, c))
    } else {
      None
    }
  }

  fn supports_cooling_profile(&self) -> bool {
    debug!("supports cooling {:?} VERSION {:?}", self.supports_cooling, self.firmware_version);
    self.supports_cooling // && self.firmware_version >= Some((3, 0, 0))
  }

  fn device_info(&self) -> &DeviceInfo {
    &self.device_info
  }

  fn read_size(&self) -> usize {
    READ_LENGTH
  }

  fn write_length(&self) -> usize {
    65
  }

  fn read_status(&mut self, buf: &[u8], read_length: usize) -> Result<DeviceStatus, DriverError> {
    if read_length != READ_LENGTH {
      Err(DriverError::DecodingError(format!(
        "Error while reading device, expected message length {} instead got {}",
        READ_LENGTH, read_length
      )))
    } else {
      let major = buf[0xb] as u16;
      let minor = (buf[0xc] as u16) << 8 | (buf[0xd] as u16);
      let build = buf[0xe] as u16;
      let firmware = (major, minor, build);
      let description = self.description.clone();
      self.firmware_version = Some(firmware.clone());
      match self.kraken_model_series {
        KrakenModelSeries::KrakenM => Ok(DeviceStatus {
          firmware: self.firmware_version,
          description,
          counters: vec![]
        }),
        KrakenModelSeries::KrakenX => {
          let liquid_temperature: f32 = (buf[1] as f32) + (buf[2] as f32) / 10.0;
          let fan_speed = (buf[3] as u16) << 8 | (buf[4] as u16);
          let pump_speed = (buf[5] as u16) << 8 | (buf[6] as u16);

          Ok(DeviceStatus {
            description,
            firmware: Some(firmware),
            counters: vec![
              Metric::new_temperature("dev.krakenX.liquid", liquid_temperature as f64, 60.0),
              Metric::new_duty("dev.krakenX.fan", fan_speed as f64, 2660.0),
              Metric::new_duty("dev.krakenX.pump", pump_speed as f64, 1760.0),
            ]
          })
        }
      }
    }
  }

  fn encode(&self, channel: &str, cfg: DeviceConf) -> Result<Vec<Vec<u8>>, DriverError> {
    match cfg {
      DeviceConf::FixedSpeed(duty) => {
        if self.supports_cooling_profile() {
          self.encode_speed_profile(channel, vec![(0, duty), (59, duty), (60, 100), (100, 100)])
        } else {
          self.encode_instantaneous_speed(channel, duty)
        }
      }
      DeviceConf::VariableSpeed(_config_for, profile) => {
        if self.supports_cooling_profile() {
          self.encode_speed_profile(channel, profile.to_vec())
        } else {
          Err(DriverError::NotSupported(format!(
            "{} incremental profiles are not supported by device",
            channel
          )))
        }
      }
    }
  }
}
