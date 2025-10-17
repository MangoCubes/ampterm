pub mod useraction;

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::action::useraction::UserAction;
use crate::osclient::response::getplaylist::Media;
use crate::playerworker::player::ToPlayerWorker;
use crate::queryworker::query::FromQueryWorker;
use crate::{app::Mode, queryworker::query::ToQueryWorker};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateType {
    Position(std::time::Duration),
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
pub struct PlayState {
    pub items: Vec<Media>,
    // Index is guaranteed to be in the range [0, items.len()]
    // In other words, items[index] may be invalid because index goes out of bound by 1
    pub index: usize,
}

impl PlayState {
    pub fn default() -> Self {
        Self {
            items: Vec::new(),
            index: 0,
        }
    }
    pub fn new(items: Vec<Media>, index: usize) -> Self {
        Self { items, index }
    }
}

/// These actions are emitted by the playerworker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FromPlayerWorker {
    // This action is used to synchronise the state of PlayerWorker with the components
    InQueue {
        play: PlayState,
        vol: f32,
        speed: f32,
        pos: Duration,
    },
    // This actions is used to send current position
    PlayerState(StateType),
    // Error sent out from the player to the components
    PlayerError(String),
    PlayerMessage(String),
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
