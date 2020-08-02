use crate::config::CoolingConf;
use crate::DeviceStatus;

pub trait Driver {
  /// Human readable description of the corresponding device
  fn description(&self) -> String;

  /// Numeric vendor identifier.
  fn vendor_id(&self) -> u16;

  /// Numeric product identifier
  fn product_id(&self) -> u16;

  /// Device versioning number, or None if N/A.
  /// In USB devices this is bcdDevice.
  fn release_number(&self) -> Option<String>;

  /// Serial number reported by the device, or None if N/A.
  fn serial_number(&self) -> Option<String> {
    None
  }

  /// Bus the device is connected to, or None if N/A.
  fn bus(&self) -> Option<String> {
    None
  }

  /// ddress of the device on the corresponding bus, or None if N/A.
  /// This typically depends on the bus enumeration order.
  fn address(&self) -> Option<String> {
    None
  }

  /// Physical location of the device, or None if N/A.
  /// This typically refers to a USB port, which is *not* dependent on bus
  /// enumeration order.  However, a USB port is hub-specific, and hubs can
  /// be chained.  Thus, for USB devices, this returns a tuple of port
  /// numbers, from the root hub to the parent of the connected device.
  fn port(&self) -> Option<String> {
    None
  }

  fn supports_cooling_profile(&self) -> bool {
    false
  }

  /// returns how much bytes should be read from device during communication
  fn read_size(&self) -> usize;

  /// returns how much bytes should be written to device during communication
  fn write_length(&self) -> usize;

  fn read_status(&mut self, buf: &[u8], read_length: usize) -> Result<DeviceStatus, DriverError>;

  fn encode(&self, cfg: CoolingConf) -> Result<Vec<&[u8]>, DriverError>;
}

#[derive(Debug)]
pub enum DriverError {
  HidApiError(String),
  NoDeviceFound,
  ReadError(String),
  EncodingError(String),
  DecodingError(String),
  NotSupported(String),
}
