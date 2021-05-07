use crate::configuration::load_configuration;
use reqwest::{get, Error as ReqwestError};

mod configuration;

#[tokio::main]
async fn main() -> Result<(), ReqwestError> {
    let configuration = load_configuration();

    let res = get(format!(
        "https://api.trello.com/1/boards/{board_id}/cards?key={key}&token={token}",
        board_id = configuration.trello.board.id,
        key = configuration.trello.api.key,
        token = configuration.trello.api.token
    ))
    .await?;

    println!("Status: {}", res.status());

    let body = res.text().await?;

    println!("Body:\n\n{}", body);

    Ok(())
}
