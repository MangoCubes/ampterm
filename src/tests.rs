use std::{sync::Arc, time::Duration};

use color_eyre::{eyre::eyre, Result};
use tokio::{
    select,
    sync::{
        mpsc::{unbounded_channel, UnboundedSender},
        RwLock,
    },
    time::sleep,
};

use crate::{
    action::action::{Action, TargetedAction},
    config::Config,
    get_audio_handle,
    playerworker::playerstatus::PlayerStatus,
    start_workers,
};

/// The actual function that sends various actions to the player
/// This function should never return before the app terminates
async fn send_keys(action_tx: UnboundedSender<Action>) -> Result<()> {
    sleep(Duration::from_secs(1)).await;
    let _ = action_tx.send(Action::Snapshot("Snap 1".to_string()));

    // Send out Quit action to the player
    let _ = action_tx.send(Action::Targeted(TargetedAction::Quit));
    // Ensure the player quits within 1 second
    sleep(Duration::from_secs(1)).await;
    Err(eyre!("Failed to quit in time!"))
}

#[tokio::test]
async fn test_main() {
    let playerstatus = Arc::from(RwLock::from(PlayerStatus::default()));
    let (action_tx, action_rx) = unbounded_channel();
    let (mpris_tx, _) = unbounded_channel();
    let (mut app, mut set) = start_workers(
        get_audio_handle(),
        action_tx.clone(),
        action_rx,
        mpris_tx,
        Config::default(),
        playerstatus,
        60.0,
        2.0,
    )
    .unwrap();
    let err = select! {
        res = send_keys(action_tx) => {
            match res {
                Ok(()) => Some("Test function somehow died??".to_string()),
                Err(e) => Some(format!("Test function failed! Error: {}", e)),
            }
        }
        res = app.run() => {
            match res {
                Ok(()) => None,
                Err(e) => Some(format!("UI panicked! Error: {}", e)),
            }
        }
        res = set.join_next() => {
            match res {
                Some(report) => match report {
                    Ok(report) => match report {
                        Ok(_) => Some("A worker has terminated itself prematurely.".to_string()),
                        Err(e) => Some(format!("A worker crashed: {}", e)),
                    },
                    Err(_) => Some("Failed to wait for the thread to run.".to_string()),
                },
                None => unreachable!("No tasks completed??"),
            }
        }
    };
    assert_eq!(err, None);
}
