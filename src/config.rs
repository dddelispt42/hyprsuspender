extern crate toml;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Config {
    default: DefaultConfig,
    presets: Option<Vec<PresetConfig>>,
}

#[derive(Debug, Deserialize)]
struct DefaultConfig {
    #[serde(default = "default_suspend_delay")]
    suspend_delay: u32,
    #[serde(default = "default_resume_every")]
    resume_every: u32,
    #[serde(default = "default_resume_for")]
    resume_for: u32,
    #[serde(default = "default_send_signals")]
    send_signals: bool,
    #[serde(default = "default_only_on_battery")]
    only_on_battery: bool,
    #[serde(default = "default_auto_suspend_on_battery")]
    auto_suspend_on_battery: bool,
    #[serde(default = "default_downclock_on_battery")]
    downclock_on_battery: u32,
}

#[derive(Debug, Deserialize)]
struct PresetConfig {
    name: String,
    #[serde(default = "default_preset_suspend_delay")]
    suspend_delay: u32,
    match_wm_class_contains: String,
    suspend_subtree_pattern: String,
}

fn default_suspend_delay() -> u32 {
    10
}
fn default_resume_every() -> u32 {
    60
}
fn default_resume_for() -> u32 {
    5
}
fn default_send_signals() -> u32 {
    true
}
fn default_only_on_battery() -> u32 {
    true
}
fn default_auto_suspend_on_battery() -> u32 {
    true
}
fn default_downclock_on_battery() -> u32 {
    true
}

impl Default for DefaultConfig {
    fn default() -> Self {
        DefaultConfig {
            suspend_delay: default_suspend_delay(),
            resume_every: default_resume_every(),
            resume_for: default_resume_for(),
            send_signals: default_send_signals(),
            only_on_battery: default_only_on_battery(),
            auto_suspend_on_battery: default_auto_suspend_on_battery(),
            downclock_on_battery: default_downclock_on_battery(),
        }
    }
}

impl Config {
    fn from_file(path: &str) -> Result<Self, toml::de::Error> {
        let mut file = File::open(path)?;
        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str)?;

        toml::from_str(&toml_str)
    }
}

fn main() {
    let config_file_path = match env::var("XDG_CONFIG_HOME") {
        Ok(xdg_config_home) => PathBuf::from(xdg_config_home).join("config.toml"),
        Err(_) => {
            eprintln!(
                "XDG_CONFIG_HOME environment variable not set, falling back to current directory."
            );
            PathBuf::from("config.toml")
        }
    };

    match Config::from_file(config_file_path.to_str().expect("Invalid path")) {
        Ok(config) => {
            println!("Default Configuration: {:?}", config.default);
            if let Some(presets) = config.presets {
                println!("Preset Configurations:");
                for preset in presets {
                    println!("{:?}", preset);
                }
            }
        }
        Err(e) => eprintln!("Error reading config file: {}", e),
    }
}

