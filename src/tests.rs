use std::{sync::Arc, time::Duration};

use color_eyre::{eyre::eyre, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
    playerworker::playerstatus::PlayerStatus,
    start_workers,
};

struct TestModule {
    action_tx: UnboundedSender<Action>,
}

impl TestModule {
    fn new(action_tx: UnboundedSender<Action>) -> Self {
        Self { action_tx }
    }

    fn send_string(&self, name: &str, text: &str) {
        let seq: Vec<KeyEvent> = text
            .chars()
            .map(|c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
            .collect();
        self.send_keys(name, seq);
    }

    fn send_key_test(&self, name: &str, key: KeyCode, modifier: KeyModifiers) {
        self.action_tx
            .send(Action::TestKey(
                Some(name.to_string()),
                KeyEvent::new(key, modifier),
            ))
            .unwrap();
    }

    fn send_key(&self, key: KeyCode, modifier: KeyModifiers) {
        self.action_tx
            .send(Action::TestKey(None, KeyEvent::new(key, modifier)))
            .unwrap();
    }

    fn send_enter(&self, name: &str) {
        self.action_tx
            .send(Action::TestKey(
                Some(name.to_string()),
                KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            ))
            .unwrap();
    }

    fn send_keys(&self, name: &str, keys: Vec<KeyEvent>) {
        self.action_tx
            .send(Action::TestKeys(name.to_string(), keys))
            .unwrap();
    }

    fn send_action(&self, action: TargetedAction) {
        self.action_tx.send(Action::Targeted(action)).unwrap();
    }

    fn snap(&self, name: &str) {
        self.action_tx
            .send(Action::Snapshot(name.to_string()))
            .unwrap();
    }

    fn test2_mainscreen(&self) {
        self.snap("Test 2a - Main Screen");
    }

    fn test1_loginscreen(&self) {
        self.snap("Test 1a - Login Screen");
        self.send_string("Test 1b", "music.local");
        self.send_key(KeyCode::Tab, KeyModifiers::NONE);
        self.send_key(KeyCode::Tab, KeyModifiers::NONE);
        self.send_string("Test 1c", "password");
        self.send_key(KeyCode::Up, KeyModifiers::NONE);
        self.send_string("Test 1d", "admin");
        self.send_key(KeyCode::Tab, KeyModifiers::SHIFT);
        self.send_key(KeyCode::Tab, KeyModifiers::SHIFT);
        self.send_key_test("Test 1e", KeyCode::Char(' '), KeyModifiers::NONE);
        self.send_enter("Test 1f");
    }

    async fn run_test(&self) -> Result<()> {
        sleep(Duration::from_secs(1)).await;
        self.test1_loginscreen();
        sleep(Duration::from_secs(1)).await;
        self.test2_mainscreen();
        sleep(Duration::from_secs(1)).await;
        // Send out Quit action to the player
        self.send_action(TargetedAction::Quit);
        // Ensure the player quits within 1 second
        // The player should quit, and take the run_test function out before it returns
        sleep(Duration::from_secs(1)).await;
        Err(eyre!("Failed to quit in time!"))
    }
}

/// The actual function that sends various actions to the player
/// This function should never return before the app terminates

#[tokio::test]
async fn test_main() {
    let playerstatus = Arc::from(RwLock::from(PlayerStatus::default()));
    let (action_tx, action_rx) = unbounded_channel();
    let (mpris_tx, _) = unbounded_channel();
    let (mut app, mut set) = start_workers(
        action_tx.clone(),
        action_rx,
        mpris_tx,
        Config::default(),
        playerstatus,
        60.0,
        2.0,
    )
    .unwrap();
    let test = TestModule::new(action_tx);
    let err = select! {
        res = test.run_test() => {
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
