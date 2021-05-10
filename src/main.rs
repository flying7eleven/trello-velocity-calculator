use crate::cli::{Opts, SubCommand};
use crate::configuration::{load_configuration, Configuration};
use clap::Clap;
use reqwest::{get, Error as ReqwestError};
use serde::{Deserialize, Serialize};

mod cli;
mod configuration;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
struct BoardLists {
    id: String,
    name: String,
}

async fn show_lists_of_board(board_id: &String, config: &Configuration) {
    if let Ok(response) = get(format!(
        "https://api.trello.com/1/boards/{board_id}/lists?key={key}&token={token}",
        board_id = board_id,
        key = config.trello.api.key,
        token = config.trello.api.token
    ))
    .await
    {
        match response.json::<Vec<BoardLists>>().await {
            Ok(available_lists) => {
                for current_list in available_lists {
                    println!(
                        "{id} => {list}",
                        id = current_list.id,
                        list = current_list.name
                    )
                }
            }
            Err(error) => println!("{}", error),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), ReqwestError> {
    // get the passed arguments as well as the configuration from the file
    let opts: Opts = Opts::parse();
    let configuration = load_configuration();

    // handle the corresponding sub-command
    match opts.subcmd {
        SubCommand::ShowListsOfBoard(show_lists) => {
            show_lists_of_board(&show_lists.board_id, &configuration).await;
        }
    }

    // if we get here, everything was okay
    Ok(())
}
