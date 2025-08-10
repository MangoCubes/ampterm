pub mod getplaylist;
pub mod getplaylists;
pub mod ping;

use std::time::Duration;

use getplaylist::{GetPlaylistResponse, Media};
use getplaylists::GetPlaylistsResponse;
use ping::PingResponse;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::playerworker::player::ToPlayerWorker;
use crate::{app::Mode, playerworker::player::QueueLocation, queryworker::query::ToQueryWorker};

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum StateType {
    Position(std::time::Duration),
    Volume(f32),
    Speed(f32),
}
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PlayState {
    pub items: Vec<Media>,
    // Index is guaranteed to be in the range [0, items.len()]
    // In other words, items[index] may be invalid because index goes out of bound by 1
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum PlayOrder {
    Normal,
    Random,
    Reverse,
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

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

/// Local actions are actions that satisfies both of these conditions:
/// 1. They are applicable in more than one mode (Insert, Normal, etc)
/// 2. Actions are applicable to the currently focused component
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum Local {
    Move(Dir),
    Confirm,
    Cancel,
    Top,
    Bottom,
    Refresh,
    ResetState,
    Help,
}

/// Visual mode exclusive actions
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum Visual {
    // Exit visual mode after applying changes
    ExitSave,
    // Exit visual mode after discarding changes
    ExitDiscard,
}

/// Insert mode exclusive actions
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum Insert {
    AddAsIs,
    Randomise,
    Reverse,
    CancelAdd,
}

/// Normal mode exclusive actions
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum Normal {
    // Action for moving between boxes
    WindowMove(Dir),
    // Enter visual mode to select items
    SelectMode,
    // Enter visual mode to deselect items
    DeselectMode,
    // Add to the queue
    Add(QueueLocation),
}

/// These actions are emitted by the playerworker.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum FromQueryWorker {
    // Responses from the queries
    Ping(PingResponse),
    GetPlaylists(GetPlaylistsResponse),
    GetPlaylist(GetPlaylistResponse),
}

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum Action {
    Suspend,
    Resume,
    Quit,
    ClearScreen,

    // Reset current component if that action is valid
    Local(Local),
    Normal(Normal),
    Visual(Visual),
    Insert(Insert),

    FromQueryWorker(FromQueryWorker),
    ToQueryWorker(ToQueryWorker),

    FromPlayerWorker(FromPlayerWorker),
    ToPlayerWorker(ToPlayerWorker),
    // This action is fired from the components to the app
    ChangeMode(Mode),

    // Anything below this should not be used for keybinds, but feel free to experiment. Most are used to notify the system
    // System actions
    Tick,
    Render,
    // ModeChanged(InputMode),
    Resize(u16, u16),
    Error(String),
    // Action for deleting all key sequences currently stored
    // It's like escape in Vim, and Ctrl+G in Emacs
    EndKeySeq,
}
