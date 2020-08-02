use core::str::FromStr;
use serde::{Deserialize, Serialize};

bitflags! {
  /// Some [ColorMode's](enum.ColorMode.html) supports animation direction, but probabbly not all,
  /// check driver docs to find out what you can use
  pub struct Direction: u8 {
    const FWD = 0x00;
    const ALT = 0x08;
    const BCK = 0x10;
  }
}

/// Configures led dynamics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", content = "params")]
pub enum ColorMode {
  /// Turs off all LEDs
  Off,
  /// Sets single color on all leds in givven channel
  Fixed(u8, u8, u8),
  /// Fades LED color. Some devices supportes multiple colors.
  Fading(Vec<(u8, u8, u8)>),
  // Complete color spectrum will be used
  // SpectrumWave(Direction),
  // LEDs will cycle throug colors so it looks like collors are
  // animated with scrolling effect
  // Marquee(Direction, Vec<Color>),
}

/// The Channel that refers to device's cooling subsystem.Iterator
///
/// For instance, some devices allow configuring `"fan"` or `"pump"`
pub type SubsystemChannel = String;

// /// Lighting color channel.
// ///
// /// Some devices supports multiple coolor channel. Default should be `"sync"`
// /// but note that this may fail or override to specific channel by driver in
// /// case where only one color channel does support specifiedd color [ColorMode](enum.ColorMode.html)`.
// pub type ColorChannel = String;

/// Animation speed
#[derive(Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Speed {
  Slowest = 0x0,
  Slow,
  Normal,
  Fast,
  Fastest,
}

impl FromStr for Speed {
  type Err = String;

  fn from_str(speed: &str) -> Result<Self, Self::Err> {
    match speed.to_lowercase().as_str() {
      "slowest" => Ok(Speed::Slowest),
      "slow" => Ok(Speed::Slow),
      "normal" => Ok(Speed::Normal),
      "fast" => Ok(Speed::Fast),
      "fastest" => Ok(Speed::Fastest),
      other => Err(format!(
        "Value '{}' is not supported animation speed.",
        other
      )),
    }
  }
}

/// The device's underling subsystem duty.
///
/// For instance, this could be **Fan** duty that can be betwee **0-100%**
pub type Duty = u8;

pub type Temperature = u8;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum MonitorHeat {
  Cpu,
  Liquid,
}

impl MonitorHeat {
  pub fn to_string(&self) -> String {
    match self {
      MonitorHeat::Cpu => "cpu".to_string(),
      _ => "liquid".to_string(),
    }
  }
}

impl Default for MonitorHeat {
  fn default() -> Self {
    MonitorHeat::Liquid
  }
}

impl<T: ToString> From<T> for MonitorHeat {
  fn from(value: T) -> Self {
    match value.to_string().to_lowercase().as_str() {
      "cpu" => MonitorHeat::Cpu,
      _ => MonitorHeat::Liquid,
    }
  }
}

/// Device Configuration
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(tag = "type", content = "params")]
pub enum CoolingConf {
  /// Fixed
  FixedSpeed(Duty),
  VariableSpeed(MonitorHeat, Vec<(Temperature, Duty)>),
}

impl Default for CoolingConf {
  fn default() -> Self {
    CoolingConf::FixedSpeed(60)
  }
}

impl CoolingConf {
  pub fn silent_fan() -> CoolingConf {
    CoolingConf::VariableSpeed(
      MonitorHeat::Liquid,
      vec![
        (20, 25),
        (25, 25),
        (30, 25),
        (35, 25),
        (40, 25),
        (45, 40),
        (50, 60),
        (55, 75),
        (60, 100),
        (65, 100),
        (70, 100),
        (75, 100),
        (80, 100),
        (85, 100),
        (90, 100),
        (95, 100),
      ],
    )
  }

  pub fn performance_fan() -> CoolingConf {
    CoolingConf::VariableSpeed(
      MonitorHeat::Liquid,
      vec![
        (20, 50),
        (25, 50),
        (30, 50),
        (35, 50),
        (40, 50),
        (45, 50),
        (50, 50),
        (55, 50),
        (60, 100),
        (65, 100),
        (70, 100),
        (75, 100),
        (80, 100),
        (85, 100),
        (90, 100),
        (95, 100),
      ],
    )
  }

  pub fn fiexed_fan() -> CoolingConf {
    CoolingConf::VariableSpeed(
      MonitorHeat::Liquid,
      (0..16).map(|v| (20 + v * 5, 25)).collect::<Vec<(u8, u8)>>(),
    )
  }

  pub fn silent_pump() -> CoolingConf {
    CoolingConf::VariableSpeed(
      MonitorHeat::Cpu,
      vec![
        (20, 50),
        (25, 50),
        (30, 50),
        (35, 50),
        (40, 50),
        (45, 50),
        (50, 50),
        (55, 100),
        (60, 100),
        (65, 100),
        (70, 100),
        (75, 100),
        (80, 100),
        (85, 100),
        (90, 100),
        (95, 100),
      ],
    )
  }

  pub fn performance_pump() -> CoolingConf {
    CoolingConf::VariableSpeed(
      MonitorHeat::Cpu,
      vec![
        (20, 70),
        (25, 70),
        (30, 70),
        (35, 70),
        (40, 80),
        (45, 80),
        (50, 80),
        (55, 80),
        (60, 100),
        (65, 100),
        (70, 100),
        (75, 100),
        (80, 100),
        (85, 100),
        (90, 100),
        (95, 100),
      ],
    )
  }

  pub fn fiexed_pump() -> CoolingConf {
    CoolingConf::VariableSpeed(
      MonitorHeat::Cpu,
      (0..16).map(|v| (20 + v * 5, 50)).collect::<Vec<(u8, u8)>>(),
    )
  }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(default)]
pub struct CoolingProfile {
  pub name: String,
  pub fan: CoolingConf,
  pub pump: CoolingConf,
}

impl Default for CoolingProfile {
  fn default() -> Self {
    CoolingProfile {
      name: "Custom".to_string(),
      fan: Default::default(),
      pump: Default::default(),
    }
  }
}
