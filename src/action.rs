pub mod useraction;

use serde::{Deserialize, Serialize};

use crate::action::useraction::UserAction;
use crate::osclient::response::getplaylist::Media;
use crate::playerworker::player::{QueueLocation, ToPlayerWorker};
use crate::queryworker::query::FromQueryWorker;
use crate::{app::Mode, queryworker::query::ToQueryWorker};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StateType {
    Position(std::time::Duration),
    NowPlaying(Option<Media>),
    Volume(f32),
    Speed(f32),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PlayOrder {
    Normal,
    Random,
    Reverse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Selection {
    Single(usize),
    Range(usize, usize),
    Multiple(Vec<bool>),
}

/// These actions are emitted by the playerworker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FromPlayerWorker {
    /// Send current state of the player
    StateChange(StateType),
    Finished,
    // Error sent out from the player to the components
    Error(String),
    Message(String),
}

/// These actions are all related to modifying the queue in one way or another.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueueAction {
    Add(Vec<Media>, QueueLocation),
}

/// FromQueryWorker enum is used to send responses from the QueryWorker to the components

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    Suspend,
    Resume,
    Quit,
    ClearScreen,

    User(UserAction),
    Queue(QueueAction),

    FromQueryWorker(FromQueryWorker),
    ToQueryWorker(ToQueryWorker),

    FromPlayerWorker(FromPlayerWorker),
    ToPlayerWorker(ToPlayerWorker),

    // Anything below this should not be used for keybinds, but feel free to experiment. Most are used to notify the system
    // System actions
    Resize(u16, u16),
    Multiple(Vec<Action>),
    // This action is fired from the components to the app
    ChangeMode(Mode),
    /// THIS IS FOR INTERNAL USE ONLY
    /// USE [`UserAction::Common(Common::EndKeySeq)`] INSTEAD
    EndKeySeq,
}
