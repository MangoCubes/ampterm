pub mod getplaylist;
pub mod getplaylists;
pub mod ping;

use getplaylist::{GetPlaylistResponse, Media};
use getplaylists::GetPlaylistsResponse;
use ping::PingResponse;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::{playerworker::player::PlayerAction, queryworker::query::Query};
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum LocalAction {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
    Top,
    Bottom,
    Refresh,
}

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    // These actions are limited to a certain focused component only
    Local(LocalAction),
    // Action for moving between boxes
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    // Action for deleting all key sequences currently stored
    // It's like escape in Vim, and Ctrl+G in Emacs
    EndKeySeq,

    // Action sent from the components to the components when a playlist is selected
    SelectPlaylist {
        key: String,
    },

    // Error sent out from the player to the components
    PlayerError(String),
    // Error sent out from the player to the components regarding data streaming
    StreamError(String),

    // Query sent from the components to the QueryWorker
    Query(Query),
    // Responses from the queries
    Ping(PingResponse),
    GetPlaylists(GetPlaylistsResponse),
    GetPlaylist(GetPlaylistResponse),

    // Actions sent from the components to the PlayerWorker
    Player(PlayerAction),
    // These are actions sent to the components, and are then sent to the PlayerWorker
    Pause,
    Play,

    // This action is used to synchronise the state of PlayerWorker with the components
    InQueue {
        current: Option<Media>,
        next: Vec<Media>,
    },
}
