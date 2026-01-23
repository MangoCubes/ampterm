use std::{sync::Arc, time::Duration};

use color_eyre::{eyre::eyre, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::{
    select,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        RwLock,
    },
    time::sleep,
};

use crate::{
    action::action::{Action, TargetedAction},
    config::{
        pathconfig::{PathConfig, PathType},
        Config,
    },
    playerworker::playerstatus::PlayerStatus,
    start_workers,
};

struct TestModule {
    action_tx: UnboundedSender<Action>,
    debug_rx: UnboundedReceiver<bool>,
    main: i32,
    sub: i32,
}

impl TestModule {
    fn new(action_tx: UnboundedSender<Action>, debug_rx: UnboundedReceiver<bool>) -> Self {
        Self {
            debug_rx,
            action_tx,
            main: 0,
            sub: 1,
        }
    }

    async fn wait_confirmation(&mut self) {
        match self.debug_rx.recv().await {
            Some(false) => panic!("Component responded with error!"),
            Some(true) => {}
            None => panic!("Debug feedback channel got closed before the test is over!"),
        }
    }

    fn new_test_section(&mut self, id: i32) {
        self.main = id;
        self.sub = 1;
    }

    fn gen_test_id(&mut self) -> String {
        let name = format!("Test {}-{:03}", self.main, self.sub);
        self.sub += 1;
        return name;
    }

    async fn send_string(&mut self, text: &str) {
        let seq: Vec<KeyEvent> = text
            .chars()
            .map(|c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
            .collect();
        self.send_keys(seq).await;
    }

    async fn send_key(&mut self, key: KeyCode, modifier: KeyModifiers) {
        self.action_tx
            .send(Action::TestKey(KeyEvent::new(key, modifier)))
            .unwrap();
        self.snap().await;
    }

    async fn send_key_skiptest(&mut self, key: KeyCode, modifier: KeyModifiers) {
        self.action_tx
            .send(Action::TestKey(KeyEvent::new(key, modifier)))
            .unwrap();
    }

    async fn send_enter(&mut self) {
        self.action_tx
            .send(Action::TestKey(KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::NONE,
            )))
            .unwrap();
        self.snap().await;
    }

    async fn send_keys(&mut self, keys: Vec<KeyEvent>) {
        self.action_tx.send(Action::TestKeys(keys)).unwrap();
        self.snap().await;
    }

    async fn send_action(&mut self, action: TargetedAction) {
        self.action_tx.send(Action::Targeted(action)).unwrap();
    }

    async fn snap(&mut self) {
        let id = self.gen_test_id();
        self.action_tx.send(Action::Snapshot(id)).unwrap();
        self.wait_confirmation().await;
    }

    async fn test3_playlistlist(&mut self) {
        self.new_test_section(3);
        self.send_key(KeyCode::Char('g'), KeyModifiers::SHIFT).await;
        self.send_key(KeyCode::Up, KeyModifiers::NONE).await;
        self.send_string("gg").await;
        self.send_key(KeyCode::Down, KeyModifiers::NONE).await;
        self.send_enter().await;
    }

    async fn test2_mainscreen(&mut self) {
        self.new_test_section(2);
        self.snap().await;
        self.send_key(KeyCode::Char('t'), KeyModifiers::NONE).await;
        self.send_key(KeyCode::Up, KeyModifiers::CONTROL).await;
        self.send_key(KeyCode::Up, KeyModifiers::CONTROL).await;
        self.send_key(KeyCode::Up, KeyModifiers::CONTROL).await;
        self.send_key(KeyCode::Up, KeyModifiers::CONTROL).await;
        self.send_key(KeyCode::Down, KeyModifiers::CONTROL).await;
        self.send_key(KeyCode::Char('t'), KeyModifiers::SHIFT).await;
        self.send_key(KeyCode::Char('t'), KeyModifiers::SHIFT).await;
        self.send_key(KeyCode::Char('t'), KeyModifiers::SHIFT).await;
        self.send_key(KeyCode::Char('?'), KeyModifiers::NONE).await;
        self.send_key(KeyCode::Left, KeyModifiers::NONE).await;
        self.send_key(KeyCode::Left, KeyModifiers::NONE).await;
        self.send_key(KeyCode::Right, KeyModifiers::NONE).await;
        self.send_key(KeyCode::Esc, KeyModifiers::NONE).await;
        self.snap().await;
    }

    async fn test1_loginscreen(&mut self) {
        self.new_test_section(1);
        self.snap().await;
        self.send_string("music.local").await;
        self.send_key_skiptest(KeyCode::Tab, KeyModifiers::NONE)
            .await;
        self.send_key_skiptest(KeyCode::Tab, KeyModifiers::NONE)
            .await;
        self.send_string("password").await;
        self.send_key_skiptest(KeyCode::Up, KeyModifiers::NONE)
            .await;
        self.send_string("admin").await;
        self.send_key_skiptest(KeyCode::Tab, KeyModifiers::SHIFT)
            .await;
        self.send_key_skiptest(KeyCode::Tab, KeyModifiers::SHIFT)
            .await;
        self.send_key(KeyCode::Char(' '), KeyModifiers::NONE).await;
        self.send_enter().await;
    }

    async fn run_test(&mut self) -> Result<()> {
        sleep(Duration::from_secs(1)).await;
        self.test1_loginscreen().await;
        sleep(Duration::from_secs(1)).await;
        self.test2_mainscreen().await;
        self.test3_playlistlist().await;
        sleep(Duration::from_secs(1)).await;
        // Send out Quit action to the player
        self.send_action(TargetedAction::Quit).await;
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
    let (debug_tx, debug_rx) = unbounded_channel();
    let (mpris_tx, _) = unbounded_channel();
    let (mut app, mut set) = start_workers(
        action_tx.clone(),
        action_rx,
        mpris_tx,
        Config::new(PathConfig::new(PathType::None, PathType::None)).unwrap(),
        playerstatus,
        60.0,
        2.0,
        debug_tx,
    )
    .unwrap();
    let mut test = TestModule::new(action_tx, debug_rx);
    set.spawn(async move { app.run().await });
    let err = select! {
        res = test.run_test() => {
            match res {
                Ok(()) => Some("Test function somehow died??".to_string()),
                Err(e) => Some(format!("Test function failed! Error: {}", e)),
            }
        }
        res = set.join_next() => {
            match res {
                Some(report) => match report {
                    Ok(report) => match report {
                        Ok(()) => None,
                        Err(e) => Some(format!("A worker crashed: {}", e)),
                    },
                    Err(e) => Some(format!("Failed to wait for the thread to run: {}", e)),
                },
                None => unreachable!("No tasks completed??"),
            }
        }
    };
    assert_eq!(err, None);
}
