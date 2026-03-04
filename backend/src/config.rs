use config_lib::{ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub(crate) struct SeverSettings {
    pub(crate) host: String,
    pub(crate) port: u16,
}

impl SeverSettings {
    pub(crate) fn get_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub(crate) struct Config {
    pub(crate) debug: bool,
    pub(crate) server: SeverSettings,
}

impl Config {
    pub(crate) fn new() -> Result<Self, ConfigError> {
        let s = config_lib::Config::builder()
            .add_source(File::with_name("config"))
            .build()?;
        s.try_deserialize()
    }
}
