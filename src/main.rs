use crate::cli::{Opts, SubCommand};
use crate::configuration::{load_configuration, Configuration};
use clap::Clap;
use cli_table::{format::Justify, print_stdout, Table, WithTitle};
use lazy_static::lazy_static;
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

#[derive(Table, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
struct CardsOfList {
    #[table(title = "ID", justify = "Justify::Left")]
    id: String,
    #[table(title = "Title", justify = "Justify::Left")]
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

async fn get_velocity_for_list(config: &Configuration, list_id: &String) -> u8 {
    use regex::Regex;

    lazy_static! {
        static ref VELOCITY_RE: Regex = Regex::new(r"^\(([1-9]{1,2})(\*{0,3})\)").unwrap();
    }

    if let Ok(response) = get(format!(
        "https://api.trello.com/1/lists/{list_id}/cards?key={key}&token={token}",
        list_id = list_id,
        key = config.trello.api.key,
        token = config.trello.api.token
    ))
    .await
    {
        match response.json::<Vec<CardsOfList>>().await {
            Ok(available_cards) => {
                let mut aggregated_velocity = 0 as u8;
                for current_card in available_cards {
                    let maybe_captures = VELOCITY_RE.captures(current_card.name.as_str());
                    if let Some(captures) = maybe_captures {
                        let velocity = &captures[1].parse::<u8>().unwrap();
                        aggregated_velocity += velocity;
                    }
                }
                return aggregated_velocity;
            }
            Err(error) => println!("{}", error),
        }
    }

    return 0;
}

async fn show_velocity(config: &Configuration) {
    let _ = print_stdout(
        vec![VelocityInformation {
            points_todo: get_velocity_for_list(config, &config.trello.lists.backlog_id).await,
            points_doing: get_velocity_for_list(config, &config.trello.lists.doing_id).await,
            points_done: get_velocity_for_list(config, &config.trello.lists.done_id).await,
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
            if let Some(supplied_board_id) = show_lists.board_id {
                show_lists_of_board(&supplied_board_id, &configuration).await;
            } else {
                if !configuration.trello.board.id.is_empty() {
                    show_lists_of_board(&configuration.trello.board.id, &configuration).await;
                } else {
                    println!("You have to supply a board id via the configuration file or the command line.")
                }
            }
        }
        SubCommand::ShowVelocity(_) => {
            show_velocity(&configuration).await;
        }
    }

    // if we get here, everything was okay
    Ok(())
}
