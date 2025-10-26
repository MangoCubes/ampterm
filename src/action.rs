pub mod useraction;

use serde::{Deserialize, Serialize};

use crate::action::useraction::UserAction;
use crate::osclient::response::getplaylist::Media;
use crate::playerworker::player::ToPlayerWorker;
use crate::queryworker::query::FromQueryWorker;
use crate::{app::Mode, queryworker::query::ToQueryWorker};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateType {
    Queue(QueueChange),
    Position(std::time::Duration),
    NowPlaying {
        music: Media,
        // Index is guaranteed to be in the range [0, items.len()]
        // In other words, items[index] may be invalid because index goes out of bound by 1
        index: usize,
    },
    Volume(f32),
    Speed(f32),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PlayOrder {
    Normal,
    Random,
    Reverse,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum QueueChange {
    Add { items: Vec<Media>, at: usize },
    Del { from: usize, to: usize },
}

impl QueueChange {
    pub fn init(items: Vec<Media>, at: usize) -> Self {
        Self::Add { items, at }
    }
    pub fn add(items: Vec<Media>, at: usize) -> Self {
        Self::Add { items, at }
    }
    pub fn del(from: usize, to: usize) -> Self {
        Self::Del { from, to }
    }
}

/// These actions are emitted by the playerworker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FromPlayerWorker {
    /// Send current state of the player
    StateChange(StateType),
    // Error sent out from the player to the components
    Error(String),
    Message(String),
}

/// FromQueryWorker enum is used to send responses from the QueryWorker to the components

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Suspend,
    Resume,
    Quit,
    ClearScreen,

    // Guarantee: All user actions are under [`UserAction`]
    User(UserAction),

    FromQueryWorker(FromQueryWorker),
    ToQueryWorker(ToQueryWorker),

    FromPlayerWorker(FromPlayerWorker),
    ToPlayerWorker(ToPlayerWorker),

    // Anything below this should not be used for keybinds, but feel free to experiment. Most are used to notify the system
    // System actions
    Tick,
    Render,
    Resize(u16, u16),
    Error(String),
    Multiple(Vec<Action>),
    // This action is fired from the components to the app
    ChangeMode(Mode),
    /// THIS IS FOR INTERNAL USE ONLY
    /// USE [`UserAction::Common(Common::EndKeySeq)`] INSTEAD
    EndKeySeq,
}
