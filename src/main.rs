use crate::cli::{Opts, SubCommand};
use crate::configuration::{load_configuration, Configuration};
use clap::Clap;
use cli_table::{format::Justify, print_stdout, Table, WithTitle};
use reqwest::{get, Error as ReqwestError};
use serde::{Deserialize, Serialize};

mod cli;
mod configuration;

#[derive(Table, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
struct BoardLists {
    #[table(title = "ID", justify = "Justify::Left")]
    id: String,
    #[table(title = "List name", justify = "Justify::Left")]
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
                let _ = print_stdout(available_lists.with_title());
            }
            Err(error) => println!("{}", error),
        }
    }
}

#[derive(Table)]
struct VelocityInformation {
    #[table(title = "Story points to do", justify = "Justify::Left")]
    points_todo: u8,
    #[table(title = "Story points doing", justify = "Justify::Left")]
    points_doing: u8,
    #[table(title = "Story points done", justify = "Justify::Left")]
    points_done: u8,
}

async fn show_velocity(_config: &Configuration) {
    print_stdout(
        vec![VelocityInformation {
            points_todo: 1,
            points_doing: 1,
            points_done: 1,
        }]
        .with_title(),
    );
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
        SubCommand::ShowVelocity(_) => {
            show_velocity(&configuration).await;
        }
    }

    // if we get here, everything was okay
    Ok(())
}
