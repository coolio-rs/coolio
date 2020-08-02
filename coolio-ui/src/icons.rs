#[derive(PartialEq, Debug, Clone)]
pub struct Icon(&'static str);

impl Icon {
  pub fn name(&self) -> &'static str {
    self.0
  }

  pub const SAVE: Icon = Icon("save-symbolic");
  pub const COPY: Icon = Icon("copy-symbolic");
  pub const TIMES_CIRCLE: Icon = Icon("times-circle-symbolic");
  pub const TRASH: Icon = Icon("trash-symbolic");
  pub const THERMOMETER_THREE_QUARTERS: Icon = Icon("thermometer-three-quarters-symbolic");
  pub const THERMOMETER_QUARTER: Icon = Icon("thermometer-quarter-symbolic");
  pub const THERMOMETER_HALF: Icon = Icon("thermometer-half-symbolic");
  pub const THERMOMETER_FULL: Icon = Icon("thermometer-full-symbolic");
  pub const THERMOMETER_EMPTY: Icon = Icon("thermometer-empty-symbolic");
  pub const TEMPERATURE_LOW: Icon = Icon("temperature-low-symbolic");
  pub const TEMPERATURE_HIGH: Icon = Icon("temperature-high-symbolic");
  pub const TINT: Icon = Icon("tint-symbolic");
  pub const WATER: Icon = Icon("water-symbolic");
  pub const FAN: Icon = Icon("fan-symbolic");
  pub const WIND: Icon = Icon("wind-symbolic");
  pub const SNOWFLAKE: Icon = Icon("snowflake-symbolic");
  pub const MICROCHIP: Icon = Icon("microchip-symbolic");
  pub const MEMORY: Icon = Icon("memory-symbolic");
  pub const DESKTOP: Icon = Icon("desktop-symbolic");
}
