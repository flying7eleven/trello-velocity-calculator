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

#[derive(Table)]
struct SprintVelocityInformation {
    #[table(title = "Sprint #", justify = "Justify::Left")]
    sprint_number: u8,
    #[table(title = "Velocity (for sprint)", justify = "Justify::Left")]
    current_velocity: u8,
    #[table(title = "Velocity (running)", justify = "Justify::Left")]
    running_velocity: f32,
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

async fn get_stored_velocities() -> Vec<SprintVelocityInformation> {
    use unqlite::{Cursor, UnQLite};

    // open the database in which we store the velocities
    let sprint_db = UnQLite::open_readonly("sprint.db");

    // a list for all entries we found
    let mut found_entries: Vec<SprintVelocityInformation> = vec![];

    // loop through the entries and collect the required information
    let mut maybe_entry = sprint_db.first();
    let mut sum_of_velocities: u16 = 0;
    while maybe_entry.is_some() {
        // get the actual entry
        let record = maybe_entry.unwrap();

        // if we have a velocity entry, add it to the list of entries
        let key = record
            .key()
            .into_iter()
            .map(|c| c as char)
            .collect::<String>();
        if key.starts_with("velocity.") {
            let current_sprint = key.replace("velocity.", "").parse::<u8>().unwrap();
            let current_velocity = record.value()[0];
            sum_of_velocities += u16::from(current_velocity);

            found_entries.push(SprintVelocityInformation {
                sprint_number: current_sprint,
                current_velocity,
                running_velocity: sum_of_velocities as f32 / (found_entries.len() + 1) as f32,
            })
        }

        // and go to the next one
        maybe_entry = record.next();
    }

    // return the found velocity entries
    found_entries
}

async fn show_stored_velocities() {
    let found_entries = get_stored_velocities().await;
    let _ = print_stdout(found_entries.with_title());
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

fn get_largest_velocity(input: &Vec<SprintVelocityInformation>) -> u32 {
    input
        .iter()
        .map(|entry| entry.current_velocity as u32)
        .max()
        .unwrap()
        + 1
}

fn get_current_velocities_as_array(input: &Vec<SprintVelocityInformation>) -> Vec<(u32, u32)> {
    input
        .iter()
        .map(|entry| (entry.sprint_number as u32, entry.current_velocity as u32))
        .collect()
}

fn get_running_velocities_as_array(input: &Vec<SprintVelocityInformation>) -> Vec<(u32, u32)> {
    input
        .iter()
        .map(|entry| (entry.sprint_number as u32, entry.running_velocity as u32))
        .collect()
}

fn get_current_running_velocity(input: &Vec<SprintVelocityInformation>) -> u32 {
    input.get(input.len() - 1).unwrap().running_velocity as u32
}

fn get_docu_text() -> String {
    format!(
        "Generated by https://github.com/flying7eleven/trello-velocity-calculator ({}.{}.{}{})",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        option_env!("CARGO_PKG_VERSION_PRE").unwrap_or("")
    )
}

async fn plot_velocity_graph(output_file_name: &String, width: u32, height: u32) {
    use plotters::prelude::*;

    // get the velocities we want to plot
    let velocity_entries = get_stored_velocities().await;

    // define the area on which we can draw our graphs
    let root_area = BitMapBackend::new(output_file_name, (width, height)).into_drawing_area();
    let _ = root_area.fill(&WHITE);
    let root_area = root_area
        .titled(
            format!(
                "Team velocity ({})",
                get_current_running_velocity(&velocity_entries)
            )
            .as_str(),
            ("sans-serif", 50.0),
        )
        .unwrap();

    // configure the chart itself (without the actual data)
    let mut chart = ChartBuilder::on(&root_area)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .caption(get_docu_text(), ("sans-serif", 11.0))
        .build_cartesian_2d(
            (1u32..(velocity_entries.len() as u32)).into_segmented(),
            0u32..get_largest_velocity(&velocity_entries),
        )
        .unwrap();

    // draw the chart boundaries
    let _ = chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .y_desc("Velocity")
        .x_desc("Sprint #")
        .axis_desc_style(("sans-serif", 15))
        .draw();

    // draw the chart data
    let _ = chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .data(get_current_velocities_as_array(&velocity_entries)),
    );

    // to avoid the IO failure being ignored silently, we manually call the present function
    let _ = root_area.present();
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
        SubCommand::ShowStoredVelocities(_) => {
            show_stored_velocities().await;
        }
        SubCommand::PlotVelocityGraph(options) => {
            plot_velocity_graph(&options.output_file_name, 1920, 1080).await;
        }
    }

    // if we get here, everything was okay
    Ok(())
}
