use reqwest::Error as ReqwestError;
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Serialize, Deserialize)]
struct ApiSettings {
    key: String,
    token: String,
}

#[derive(Serialize, Deserialize)]
struct TrelloSettings {
    api: ApiSettings,
}

#[derive(Serialize, Deserialize)]
struct Configuration {
    trello: TrelloSettings,
}

fn load_configuration() -> Configuration {
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
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), ReqwestError> {
    let _configuration = load_configuration();

    Ok(())
}
