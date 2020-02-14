use serde::{
    Deserialize,
    Serialize,
};

use std::{
    fs,
    io,
    sync,
    path::Path,
};

pub mod matches;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchRule {
    Executable {
        name: String
    },
    AppInfo {
        name: Option<String>,
        engine: Option<String>,
    },
    And {
        rules: Vec<Box<MatchRule>>
    },
    Or {
        rules: Vec<Box<MatchRule>>
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    match_rule: MatchRule,
    filter: String,
}

impl Filter {
    #[inline(always)]
    pub fn match_rule(&self) -> &MatchRule {
        &self.match_rule
    }

    #[inline(always)]
    pub fn filter(&self) -> &str {
        self.filter.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    filters: Vec<Filter>,
}

#[inline]
fn open_config<P>(path: P) -> io::Result<fs::File>
where
    P: AsRef<Path>,
{
    fs::OpenOptions::new()
        .read(true)
        .open(path)
}

static INIT_CONFIG: sync::Once = sync::Once::new();
static mut CONFIG: Option<Config> = None;

impl Config {
    pub fn global() -> &'static Self {
        unsafe {
            INIT_CONFIG.call_once(|| {
                let cfg = Config::read()
                    .expect("Failed to read config");
                #[cfg(debug)]
                {
                    println!("config: {:?}", &cfg);
                }
                CONFIG = Some(cfg);
            });
            CONFIG.as_ref().unwrap()
        }
    }

    #[inline]
    pub fn filters<'a>(&'a self) -> impl Iterator<Item=&'a Filter> {
        self.filters.iter()
    }

    fn read() -> Result<Self, serde_yaml::Error> {
        use std::env;
        if let Some(config_file) = env::var("VK_DEVICE_FILTER_CONFIG").ok() {
            // In the case of a specifically specified config, we want this to be the only config
            // and we want to panic if it doesn't exist
            let file = open_config(config_file)
                .expect("Failed to open environment-specified config file");
            return serde_yaml::from_reader(file);
        }
        let mut config = Config::new();
        if let Some(mut config_path) = dirs::config_dir() {
            config_path.push("vulkan-device-filter/config.yml");
            if let Ok(file) = open_config(&config_path) {
                let cfg = serde_yaml::from_reader(file)?;
                config.merge(cfg);
            }
        }
        #[cfg(target_os = "linux")]
        {
            if let Ok(file) = open_config("/etc/vulkan-device-filter/config.yml") {
                let cfg = serde_yaml::from_reader(file)?;
                config.merge(cfg);
            }
            if let Ok(file) = open_config("/usr/share/vulkan-device-filter/config.yml") {
                let cfg = serde_yaml::from_reader(file)?;
                config.merge(cfg);
            }
        }
        Ok(config)
    }

    #[inline]
    fn new() -> Self {
        Config {
            filters: Vec::new(),
        }
    }

    fn merge(&mut self, other: Config) {
        self.filters.extend(other.filters);
    }
}
