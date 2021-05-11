use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Serialize, Deserialize)]
pub struct ApiSettings {
    pub key: String,
    pub token: String,
}

impl Default for ApiSettings {
    fn default() -> Self {
        ApiSettings {
            key: "".to_string(),
            token: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct BoardSettings {
    pub id: String,
}

impl Default for BoardSettings {
    fn default() -> Self {
        BoardSettings { id: "".to_string() }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ListSettings {
    pub backlog_id: String,
    pub doing_id: String,
    pub done_id: String,
}

impl Default for ListSettings {
    fn default() -> Self {
        ListSettings {
            backlog_id: "".to_string(),
            doing_id: "".to_string(),
            done_id: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TrelloSettings {
    pub api: ApiSettings,
    pub board: BoardSettings,
    pub lists: ListSettings,
}

impl Default for TrelloSettings {
    fn default() -> Self {
        TrelloSettings {
            api: ApiSettings::default(),
            board: BoardSettings::default(),
            lists: ListSettings::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    pub trello: TrelloSettings,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            trello: TrelloSettings::default(),
        }
    }
}

pub fn load_configuration() -> Configuration {
    // try to open the configuration of the program...
    if let Ok(file) = File::open("velocity.toml") {
        // ... and read the whole content of the file into memory
        let mut contents = String::new();
        let mut file_reader = BufReader::new(file);
        if let Ok(_) = file_reader.read_to_string(&mut contents) {
            // if the configuration can be deserialized, return the deserialized object
            if let Ok(config) = toml::from_str::<Configuration>(&contents) {
                return config;
            }
        }
    }

    // if we reach this step, we could not read any configuration file and return an empty one
    Configuration::default()
}
