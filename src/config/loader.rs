use super::schema::Config;
use crate::error::{ConfigError, ConfigResult};
use std::fs;
use std::path::{Path, PathBuf};

pub fn find_config_files() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let project_config = PathBuf::from("./copyconfig.toml");
    if project_config.exists() {
        paths.push(project_config);
    }
    if let Some(config_dir) = dirs::config_dir() {
        let user_config = config_dir.join("copy").join("copyconfig.toml");
        if user_config.exists() {
            paths.push(user_config);
        }
    }
    #[cfg(unix)]
    {
        let system_config = PathBuf::from("/etc/copy/copyconfig.toml");
        if system_config.exists() {
            paths.push(system_config);
        }
    }
    paths
}

pub fn load_config_file(path: &Path) -> ConfigResult<Config> {
    let contents = fs::read_to_string(path).map_err(ConfigError::Io)?;
    let config: Config = toml::from_str(&contents).map_err(ConfigError::Parse)?;
    Ok(config)
}

/// Load and merge all config files (reverse priority: system < user < project)
pub fn load_config() -> Config {
    let project = PathBuf::from("./copyconfig.toml");
    if project.exists()
        && let Ok(config) = load_config_file(&project)
    {
        return config;
    }

    if let Some(config_dir) = dirs::config_dir() {
        let user = config_dir.join("copy").join("copyconfig.toml");
        if user.exists()
            && let Ok(config) = load_config_file(&user)
        {
            return config;
        }
    }

    #[cfg(unix)]
    {
        let system = PathBuf::from("/etc/copy/copyconfig.toml");
        if system.exists()
            && let Ok(config) = load_config_file(&system)
        {
            return config;
        }
    }

    Config::default()
}
