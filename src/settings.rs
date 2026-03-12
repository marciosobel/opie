use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {}

impl Settings {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load() -> Result<Self> {
        let path = Self::path();
        if !path.exists() {
            let settings = Self::new();
            settings.save()?; // Write the settings file for the first time
            return Ok(settings);
        }

        let serialized = std::fs::read_to_string(path)?;
        let settings: Self = serde_json::from_str(&serialized)?;
        Ok(settings)
    }

    pub fn path() -> std::path::PathBuf {
        dirs::config_dir()
            .expect("Settings folder not found!")
            .join("opie/settings.json")
    }

    pub fn save(&self) -> Result<()> {
        let serialized = serde_json::to_string_pretty(self)?;

        let path = Self::path();
        if !path.exists() {
            let parent = path
                .parent()
                .context("Settings folder parent does not exist")?;

            std::fs::create_dir_all(parent).context("Was not able to create file")?;
        }

        std::fs::write(path, serialized)?;
        Ok(())
    }
}
