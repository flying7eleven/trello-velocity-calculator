use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Serialize, Deserialize)]
pub struct ApiSettings {
    pub key: String,
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct BoardSettings {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListSettings {
    pub backlog_id: String,
    pub doing_id: String,
    pub done_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct TrelloSettings {
    pub api: ApiSettings,
    pub board: BoardSettings,
    pub lists: ListSettings,
}

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    pub trello: TrelloSettings,
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
    Configuration {
        trello: TrelloSettings {
            api: ApiSettings {
                key: "".to_string(),
                token: "".to_string(),
            },
            board: BoardSettings { id: "".to_string() },
            lists: ListSettings {
                backlog_id: "".to_string(),
                doing_id: "".to_string(),
                done_id: "".to_string(),
            },
        },
    }
}
