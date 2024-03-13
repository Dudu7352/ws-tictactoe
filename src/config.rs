use serde::Deserialize;

#[derive(Debug)]
pub enum ConfigLoadError {
    CannotAccessFile,
    InavlidConfig,
}

#[derive(Deserialize)]
pub struct Config {
    // TODO: host_static, allow_cors
}

impl Config {
    pub fn load() -> Result<Self, ConfigLoadError> {
        let file_conent = std::fs::read_to_string("config.yaml")
            .map_err(|_| ConfigLoadError::CannotAccessFile)?;
        serde_yaml::from_str(&file_conent).map_err(|_| ConfigLoadError::InavlidConfig)
    }
}
