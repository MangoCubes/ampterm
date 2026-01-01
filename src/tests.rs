#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use color_eyre::Result;
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
        config::Config,
        get_audio_handle,
        playerworker::playerstatus::PlayerStatus,
        start_workers,
        tui::{Event, Tui},
    };

    async fn send_keys(event_tx: UnboundedSender<Event>) -> Result<()> {
        let _ = event_tx.send(Event::Key(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::NONE,
        )));
        sleep(Duration::from_secs(1)).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_main() {
        let playerstatus = Arc::from(RwLock::from(PlayerStatus::default()));
        let (action_tx, action_rx) = unbounded_channel();
        let (mpris_tx, _) = unbounded_channel();
        let tui = Tui::new()
            .unwrap()
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(2.0)
            .frame_rate(60.0);
        let backend = tui.backend();
        let event_tx = tui.event_tx.clone();
        let (mut app, mut set) = start_workers(
            get_audio_handle(),
            action_tx,
            action_rx,
            mpris_tx,
            Config::default(),
            playerstatus,
            tui,
        )
        .unwrap();
        let err = select! {
            res = send_keys(event_tx) => {
                match res {
                    Ok(()) => None,
                    Err(e) => Some(format!("Test function failed! Error: {}", e)),
                }
            }
            res = app.run() => {
                match res {
                    Ok(()) => Some("UI has terminated itself prematurely.".to_string()),
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
        if let Some(err) = err {
            panic!("{}", err);
        }
    }
}
