use serde::Deserialize;

#[derive(Debug)]
pub enum ConfigLoadError {
    CannotAccessFile,
    InavlidConfig,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub host_static: bool,
    #[serde(default)]
    pub cors_origin: Option<String>
}

impl Config {
    pub fn load() -> Result<Self, ConfigLoadError> {
        let file_conent = std::fs::read_to_string("config.yaml")
            .map_err(|_| ConfigLoadError::CannotAccessFile)?;
        serde_yaml::from_str(file_conent.as_str()).map_err(|_| ConfigLoadError::InavlidConfig)
    }
}
