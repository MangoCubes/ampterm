use serde::{Deserialize, Serialize};

use crate::{
    osclient::response::getplaylist::Media,
    playerworker::player::{FromPlayerWorker, QueueLocation, ToPlayerWorker},
    queryworker::query::{FromQueryWorker, ToQueryWorker},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    Normal,
    Visual,
    Insert,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        match &self {
            Mode::Normal => "NORMAL".to_string(),
            Mode::Visual => "VISUAL".to_string(),
            Mode::Insert => "INSERT".to_string(),
        }
    }
}

/// These actions are all related to modifying the queue in one way or another.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueueAction {
    Add(Vec<Media>, QueueLocation),
    RandomAdd(Vec<Media>, QueueLocation),
}

impl ToString for QueueAction {
    fn to_string(&self) -> String {
        match self {
            QueueAction::Add(_, queue_location) => match queue_location {
                QueueLocation::Front => "Play selected items immediately",
                QueueLocation::Next => "Play selected items next",
                QueueLocation::Last => "Append selected items to the end of the queue",
            },
            QueueAction::RandomAdd(_, queue_location) => match queue_location {
                QueueLocation::Front => "Shuffle the selected items and play them immediately",
                QueueLocation::Next => "Shuffle the selected items and play them next",
                QueueLocation::Last => {
                    "Shuffle the selected items and add them to the end of the queue"
                }
            },
        }
        .to_string()
    }
}

/// These actions are associated with a specific component in the program, and are usually
/// available regardles of the currently focused component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TargetedAction {
    Play,
    Pause,
    PlayOrPause,
    Skip,
    Previous,
    Queue(QueueAction),
    GoToStart,
    ChangeVolume(f32),
    ChangeSpeed(f32),
    SetVolume(f32),
    SetSpeed(f32),
    ChangePosition(f32),

    // Action for moving between boxes
    WindowUp,
    WindowDown,
    WindowLeft,
    WindowRight,

    TapToBPM,
    FocusPlaylistList,
    FocusPlaylistQueue,
    FocusPlayQueue,
    OpenTasks,
    CloseTasks,
    ToggleTasks,

    EndKeySeq,

    OpenHelp,
    CloseHelp,
    ToggleHelp,

    Suspend,
    Resume,
    ClearScreen,
    Quit,
}

impl ToString for TargetedAction {
    fn to_string(&self) -> String {
        match self {
            TargetedAction::Play => "Play music".to_string(),
            TargetedAction::Pause => "Pause music".to_string(),
            TargetedAction::PlayOrPause => "Play/Pause".to_string(),
            TargetedAction::Skip => "Skip to next music".to_string(),
            TargetedAction::Previous => "Skip to previous music".to_string(),
            TargetedAction::Queue(q) => q.to_string(),
            TargetedAction::GoToStart => "Rewind to start".to_string(),
            TargetedAction::ChangeVolume(v) => {
                if *v >= 0.0 {
                    format!("Increase volume by {}", v)
                } else {
                    format!("Decrease volume by {}", -v)
                }
            }
            TargetedAction::WindowUp => "Focus window above".to_string(),
            TargetedAction::WindowDown => "Focus window below".to_string(),
            TargetedAction::WindowLeft => "Focus window on the left".to_string(),
            TargetedAction::WindowRight => "Focus window on the right".to_string(),
            TargetedAction::TapToBPM => "Tap to BPM".to_string(),
            TargetedAction::FocusPlaylistList => "Focus playlist list".to_string(),
            TargetedAction::FocusPlaylistQueue => "Focus playlist queue".to_string(),
            TargetedAction::FocusPlayQueue => "Focus play queue".to_string(),
            TargetedAction::OpenTasks => "Open tasks view".to_string(),
            TargetedAction::CloseTasks => "Close tasks view".to_string(),
            TargetedAction::ToggleTasks => "Toggle tasks view".to_string(),
            TargetedAction::EndKeySeq => "Reset key sequence".to_string(),
            TargetedAction::OpenHelp => "Open help menu".to_string(),
            TargetedAction::CloseHelp => "Close help menu".to_string(),
            TargetedAction::ToggleHelp => "Toggle help menu".to_string(),
            TargetedAction::Suspend => "Suspend program".to_string(),
            TargetedAction::Resume => "Resume program".to_string(),
            TargetedAction::ClearScreen => "Re-render".to_string(),
            TargetedAction::Quit => "Quit program".to_string(),
            TargetedAction::ChangeSpeed(s) => {
                if *s >= 0.0 {
                    format!("Increase playback speed by {}", s)
                } else {
                    format!("Decrease playback speed by {}", -s)
                }
            }
            TargetedAction::SetVolume(v) => format!("Set volume to {}", v),
            TargetedAction::SetSpeed(v) => format!("Set playback speed to {}", v),
            TargetedAction::ChangePosition(p) => {
                if *p >= 0.0 {
                    format!("Seek forward {} seconds", p)
                } else {
                    format!("Seek backwards {} seconds", -p)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryAction {
    /// Handle response from the player worker in response to the previous request
    FromPlayerWorker(FromPlayerWorker),
    /// Receive a response from the query worker in response to the previous request
    FromQueryWorker(FromQueryWorker),
    ToPlayerWorker(ToPlayerWorker),
    ToQueryWorker(ToQueryWorker),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Multiple(Vec<Action>),
    Targeted(TargetedAction),
    Resize(u16, u16),
    ChangeMode(Mode),
    Query(QueryAction),
    Render,
}
