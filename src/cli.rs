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
    ShowCurrentVelocity(ShowCurrentVelocity),
    AddSprintVelocity(AddSprintVelocity),
    AddSprintVelocityManually(AddSprintVelocityManually),
}

/// This sub-command can be used to show the available lists of a specific board (for the initial configuration)
#[derive(Clap)]
pub struct ShowListsOfBoard {
    /// The board id for which the available lists should be displayed
    pub board_id: Option<String>,
}

/// This sub-command can be used to show the velocity of the current sprint
#[derive(Clap)]
pub struct ShowCurrentVelocity {}

/// This sub-command can be used to store the velocity for the current sprint
#[derive(Clap)]
pub struct AddSprintVelocity {}

/// This sub-command can be used to store the velocity of a  sprint manually
#[derive(Clap)]
pub struct AddSprintVelocityManually {
    /// The number which identifies the sprint
    pub sprint_number: u8,
    /// The number of velocity points finished in the given sprint
    pub velocity: u8,
}
