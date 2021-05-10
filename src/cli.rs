use clap::{AppSettings, Clap};

/// This tool can be used to calculate the velocity for a SCRUM team based on the
/// voted stories on a Trello board.
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    ShowListsOfBoard(ShowListsOfBoard),
}

/// This sub-command can be used to show the available lists of a specific board (for the initial configuration)
#[derive(Clap)]
pub struct ShowListsOfBoard {
    /// The board id for which the available lists should be displayed
    pub board_id: String,
}
