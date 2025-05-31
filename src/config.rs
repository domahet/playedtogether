use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use dirs; // For finding config directory

use crate::riot_id::{RiotId}; // Import RiotId from our new module

const CONFIG_FILE_NAME: &str = "config.json";

/// Helper struct for `RiotId` to be `Serializable` and `Deserializable`
/// because `FromStr` and `Display` traits aren't directly compatible with Serde.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiotIdSerializable {
    pub game_name: String,
    pub tag_line: String,
}

impl From<RiotId> for RiotIdSerializable {
    fn from(riot_id: RiotId) -> Self {
        RiotIdSerializable {
            game_name: riot_id.game_name,
            tag_line: riot_id.tag_line,
        }
    }
}

impl From<RiotIdSerializable> for RiotId {
    fn from(riot_id_s: RiotIdSerializable) -> Self {
        RiotId {
            game_name: riot_id_s.game_name,
            tag_line: riot_id_s.tag_line,
        }
    }
}

/// Configuration structure to store the "self" Riot ID.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub self_riot_id: Option<RiotIdSerializable>,
    pub api_key: Option<String>,
}

impl Config {
    /// Gets the path to the configuration file.
    fn config_file_path() -> Result<PathBuf, Box<dyn Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not find configuration directory.")?
            .join("rito"); // Use your tool's name as a subdirectory
        fs::create_dir_all(&config_dir)?; // Create the directory if it doesn't exist
        Ok(config_dir.join(CONFIG_FILE_NAME))
    }

    /// Loads the configuration from the file.
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let path = Self::config_file_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Config::default())
        }
    }

    /// Saves the configuration to the file.
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let path = Self::config_file_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }
}