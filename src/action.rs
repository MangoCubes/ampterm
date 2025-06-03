pub mod getplaylist;
pub mod getplaylists;
pub mod ping;

use std::time::Duration;

use getplaylist::{GetPlaylistResponse, Media};
use getplaylists::GetPlaylistsResponse;
use ping::PingResponse;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::{playerworker::player::PlayerAction, queryworker::query::Query};
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum StateType {
    Position(std::time::Duration),
    Volume(f32),
    Speed(f32),
}

// Macro for getting actions that are sent to the currently focused component only
#[macro_export]
macro_rules! local_action {
    () => {
        Action::Up
            | Action::Down
            | Action::Right
            | Action::Left
            | Action::Confirm
            | Action::Cancel
            | Action::Top
            | Action::Bottom
            | Action::Refresh
            | Action::AddFront
            | Action::AddNext
            | Action::AddLast
    };
}

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum Action {
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Help,
    // Action for moving between boxes
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    // Action for deleting all key sequences currently stored
    // It's like escape in Vim, and Ctrl+G in Emacs
    EndKeySeq,
    // Play or pause the playerworker
    Pause,
    Play,
    Skip,

    VisualMode,
    ExitMode,

    // Movement-related actions
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
    Top,
    Bottom,
    Refresh,
    // Add selected elements to the queue
    AddFront,
    AddNext,
    AddLast,
    // Anything below this should not be used for keybinds
    // System actions
    Tick,
    Render,
    Resize(u16, u16),
    Error(String),
    // Action sent from the components to the components when a playlist is selected
    SelectPlaylist {
        key: String,
    },

    // Error sent out from the player to the components
    PlayerError(String),
    PlayerMessage(String),

    // Query sent from the components to the QueryWorker
    Query(Query),
    // Responses from the queries
    Ping(PingResponse),
    GetPlaylists(GetPlaylistsResponse),
    GetPlaylist(GetPlaylistResponse),

    // Actions sent from the components to the PlayerWorker
    Player(PlayerAction),

    // This action is used to synchronise the state of PlayerWorker with the components
    InQueue {
        current: Option<Media>,
        next: Vec<Media>,
        vol: f32,
        speed: f32,
        pos: Duration,
    },
    // This actions is used to send current position
    PlayerState(StateType),
}
