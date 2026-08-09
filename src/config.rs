use std::fs;

use eros::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_launch_fetch")]
    pub launch_fetch: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            launch_fetch: Self::default_launch_fetch(),
        }
    }
}

impl Config {
    pub fn load_or_default() -> eros::Result<Self> {
        let mut file = dirs::config_dir()
            .expect("the operating system should provide a user config directory");
        file.extend(["jj-bond", "config.toml"]);

        match file.exists() {
            true => {
                let content = fs::read(&file).with_context(|| {
                    format!("failed at reading config file `{}`", file.display())
                })?;
                let v: Self = toml::from_slice(&content).with_context(|| {
                    format!("failed at parsing config file `{}`", file.display())
                })?;
                Ok(v)
            }
            false => {
                let dir = file.parent().expect(
                    "config file path should have a parent because it is built from config_dir",
                );
                fs::create_dir_all(dir).with_context(|| {
                    format!("failed at creating config directory `{}`", dir.display())
                })?;
                fs::write(
                    &file,
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/templates/config.toml"
                    )),
                )
                .with_context(|| {
                    format!("failed at writing default config file `{}`", file.display())
                })?;
                Ok(Self::default())
            }
        }
    }
}

impl Config {
    fn default_launch_fetch() -> bool {
        true
    }
}
