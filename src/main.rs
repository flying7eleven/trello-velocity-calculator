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

async fn get_last_sprint_number() -> u8 {
    use unqlite::{UnQLite, KV};

    // open the database in which we store the sprint number
    let sprint_db = UnQLite::open_readonly("sprint.db");

    // if there is a value stored, return the last sprint number
    if let Ok(value) = sprint_db.kv_fetch("last_sprint_number") {
        return value[0];
    }

    // if seems there was no sprint info stored so far, the last sprint is assumed to be
    // zero
    0
}

async fn store_sprint_information(sprint_number: u8, finished_story_points: u8) {
    use unqlite::{Transaction, UnQLite, KV};

    // get the last known sprint number
    let last_stored_sprint_number = get_last_sprint_number().await;

    // open the database where we store everything and open a transaction
    let sprint_db = UnQLite::create("sprint.db");
    let _ = sprint_db.begin();

    // just update the last stored sprint number, if it is a new one
    if last_stored_sprint_number < sprint_number {
        let _ = sprint_db.kv_store("last_sprint_number", vec![sprint_number]);
    }

    // write the velocity of the sprint
    let _ = sprint_db.kv_store(
        format!("velocity.{}", sprint_number),
        vec![finished_story_points],
    );

    // finish the transaction
    let _ = sprint_db.commit();
}

async fn ask_yes_no_question(question: String) -> bool {
    use ncurses::{addstr, endwin, getch, initscr, refresh};

    // initialize the library for interacting with the user
    let mut user_response = false;
    initscr();

    // keep asking the user until we get a yes or no answer
    loop {
        addstr(format!("{} [y/n]", question).as_str());
        refresh();

        match getch() as u8 as char {
            'Y' | 'y' => user_response = true,
            'N' | 'n' => user_response = false,
            _ => {
                addstr("\n");
                continue;
            }
        }
        break;
    }

    // clean up and return the result
    refresh();
    endwin();
    user_response
}

async fn query_and_store_sprint_velocity(config: &Configuration) {
    let finished_story_points = get_velocity_for_list(config, &config.trello.lists.done_id).await;
    let assumed_sprint_number = get_last_sprint_number().await + 1;

    store_sprint_velocity(assumed_sprint_number, finished_story_points).await;
}

async fn store_sprint_velocity(assumed_sprint_number: u8, finished_story_points: u8) {
    // ask the user if the determined information are correct or not
    let information_correct = ask_yes_no_question(format!(
        "Is it correct that you finished {} velocity point(s) in your {}. sprint?",
        finished_story_points, assumed_sprint_number
    ))
    .await;

    // if the information are not correct, skip the storing of the information
    if !information_correct {
        println!("Aborting!");
        return;
    }

    // store the information about the sprint in the database
    store_sprint_information(assumed_sprint_number, finished_story_points).await;
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
        SubCommand::ShowCurrentVelocity(_) => {
            show_velocity(&configuration).await;
        }
        SubCommand::AddSprintVelocity(_) => {
            query_and_store_sprint_velocity(&configuration).await;
        }
        SubCommand::AddSprintVelocityManually(sprint_velocity_infos) => {
            store_sprint_velocity(
                sprint_velocity_infos.sprint_number,
                sprint_velocity_infos.velocity,
            )
            .await;
        }
    }

    // if we get here, everything was okay
    Ok(())
}
