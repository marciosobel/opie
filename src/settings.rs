use std::{fs, path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {}

impl Settings {
    /// Loads and parses the settings from the settings file.
    pub fn load() -> Result<Self> {
        let path = Self::path();
        if !path.exists() {
            let settings = Self::default();
            // Write the settings file for the first time
            settings.save()?;
            return Ok(settings);
        }

        let serialized = fs::read_to_string(path)?;
        let settings: Self = serde_json::from_str(&serialized)?;
        Ok(settings)
    }

    /// Gets the default settings `path`.
    pub fn path() -> path::PathBuf {
        dirs::config_dir()
            .expect("Settings folder not found!")
            .join("opie/settings.json")
    }

    /// Saves the current settings into the settings file.
    pub fn save(&self) -> Result<()> {
        let serialized = serde_json::to_string_pretty(self)?;

        let path = Self::path();
        if !path.exists() {
            let parent = path
                .parent()
                .context("Settings folder parent does not exist")?;

            fs::create_dir_all(parent).context("Was not able to create file")?;
        }

        fs::write(path, serialized)?;
        Ok(())
    }
}
