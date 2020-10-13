use coolio_drivers::{DeviceConf, CoolingProfile};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(default)]
pub struct AppConfig {
  pub selected_profile: Option<String>,
  pub profiles: Vec<CoolingProfile>,
}

impl Default for AppConfig {
  fn default() -> Self {
    AppConfig {
      selected_profile: Some("Silent".to_string()),
      profiles: vec![
        CoolingProfile {
          name: String::from("Silent"),
          fan: DeviceConf::silent_fan(),
          pump: DeviceConf::silent_pump(),
        },
        CoolingProfile {
          name: String::from("Performance"),
          fan: DeviceConf::performance_fan(),
          pump: DeviceConf::performance_pump(),
        },
        CoolingProfile {
          name: String::from("Fixed"),
          fan: DeviceConf::fixed_fan(),
          pump: DeviceConf::fiexed_pump(),
        },
      ],
    }
  }
}

impl AppConfig {
  pub fn to_abs_path(app_path_relative: &str) -> String {
    let app_dir = std::env::current_exe().expect("App dir is unknown");
    let pathbuf = std::path::Path::new(&app_dir).join(app_path_relative);
    pathbuf.to_str().expect("ah, path is invalid").to_string()
  }

  pub fn current(&mut self) -> CoolingProfile {
    let current = if let Some(selected) = &self.selected_profile {
      self.profiles.iter().find(|p| &p.name == selected)
    } else {
      None
    };

    if current.is_none() {
      // probably will never occur in app, but lets cover the case
      let new_profile: CoolingProfile = Default::default();
      let name = new_profile.name.clone();
      self.profiles.push(new_profile);
      self.selected_profile = Some(name);
      self.profiles.last().unwrap().clone()
    } else {
      current.unwrap().clone()
    }
  }

  #[allow(dead_code)]
  pub fn load() -> Self {
    std::env::var("HOME")
      .map(|home| {
        let mut pbuff = std::path::PathBuf::from(home);
        pbuff.push(".coolio");
        pbuff.push("config.toml");
        let path = std::path::Path::new(&pbuff);
        if let Ok(cfg_str) = std::fs::read_to_string(path) {
          toml::from_str(&cfg_str).unwrap_or_default()
        } else {
          Default::default()
        }
      })
      .unwrap_or_default()
  }

  #[allow(dead_code)]
  pub fn save<'a>(&mut self) -> Result<(), &'a str> {
    let home = std::env::var("HOME").or(Err("HOME env variable is not set"))?;
    let mut path = PathBuf::from(home);
    path.push(".coolio");
    fs::create_dir_all(path.clone()).or(Err("Failed to create folder .coolio in HOME"))?;
    path.push("config.toml");
    let path = Path::new(&path);
    let buff: &[u8] = &toml::to_vec(self).or(Err("Error while serializing configuration"))?;

    let mut file = fs::File::create(path).or(Err("error while trying to open config file"))?;

    file
      .write_all(buff)
      .or(Err("Error while serializing configuration"))?;
    Ok(())
  }
}

#[cfg(test)]
mod config_test {
  use crate::config::AppConfig;
  use coolio_drivers::DeviceConf;
  use coolio_drivers::CoolingProfile;
  use std::env;
  use std::path::{Path, PathBuf};

  #[test]
  fn should_save_config_to_user_home_holder() {
    let home = env::var("HOME").expect("No HOME env var");
    let mut path = PathBuf::from(home);
    path.push(".coolio");
    path.push("config.toml");

    let mut cfg = AppConfig {
      selected_profile: Some("ConfigTest".to_string()),
      profiles: vec![CoolingProfile {
        name: "ConfigTest".to_string(),
        fan: DeviceConf::FixedSpeed(20),
        pump: DeviceConf::FixedSpeed(60),
      }],
    };

    assert_eq!(Ok(()), cfg.save());
    assert_eq!(true, Path::new(&path).is_file());

    let saved = AppConfig::load();
    assert_eq!(saved, cfg);
  }

  #[test]
  fn should_serialize_simple_profile() {
    let config = toml::to_string(&AppConfig {
      selected_profile: Some("Fixed".to_string()),
      profiles: vec![CoolingProfile {
        name: "Fixed".to_string(),
        fan: DeviceConf::FixedSpeed(20),
        pump: DeviceConf::FixedSpeed(60),
      }],
    })
    .unwrap();
    assert_eq!(
      config,
      r#"selected_profile = "Fixed"

[[profiles]]
name = "Fixed"

[profiles.fan]
type = "FixedSpeed"
params = 20

[profiles.pump]
type = "FixedSpeed"
params = 60
"#
    );
  }

  #[test]
  fn should_load_config() {
    let config: AppConfig = toml::from_str(
      r#"[[profiles]]
name = "Fixed"
current = false

  [profiles.fan]
  type = "FixedSpeed"
  params = 20

[[profiles]]
name = "Fixed2"
current = false

  [profiles.fan]
  type = "FixedSpeed"
  params = 20

  [profiles.pump]
  type = "FixedSpeed"
  params = 60
"#,
    )
    .unwrap();
    assert_eq!(config.profiles[0].name, "Fixed");
    assert_eq!(config.profiles[1].name, "Fixed2");
  }
}
