use std::sync::Arc;

use clap::Parser;
use cli::Cli;
use color_eyre::{eyre::eyre, Result};
use mpris_server::Server;
use tokio::{
    select,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        RwLock,
    },
    task::JoinSet,
};
use tracing::warn;

use crate::{
    action::action::Action,
    app::App,
    config::{
        pathconfig::{PathConfig, PathType},
        Config,
    },
    mpris::AmptermMpris,
    playerworker::{player::FromPlayerWorker, playerstatus::PlayerStatus, PlayerWorker},
    queryworker::QueryWorker,
};

mod action;
mod app;
mod cli;
mod compid;
mod components;
mod config;
mod errors;
mod helper;
mod logging;
mod lyricsclient;
mod mpris;
mod osclient;
mod playerworker;
mod queryworker;
#[cfg(test)]
mod tests;
mod tui;

pub fn log_alsa_error() {
    #[cfg(target_os = "linux")]
    {
        unsafe extern "C" fn error_handler(
            file: *const ::std::os::raw::c_char,
            line: ::std::os::raw::c_int,
            function: *const ::std::os::raw::c_char,
            err: ::std::os::raw::c_int,
            fmt: *const ::std::os::raw::c_char,
            // mut args: ...
        ) {
            let file_str = unsafe { std::ffi::CStr::from_ptr(file).to_string_lossy() };
            let function_str = unsafe { std::ffi::CStr::from_ptr(function).to_string_lossy() };
            let fmt_str = unsafe { std::ffi::CStr::from_ptr(fmt).to_string_lossy() };
            warn!("{file_str}:{line} {function_str} {err} {fmt_str}");
        }
        // For some reason, buffer underruning glitch occurs and I have no idea how to fix that :(
        // It causes the playback to stop briefly.
        // For now, the error message is suppressed.
        unsafe {
            let handler: alsa_sys::snd_lib_error_handler_t =
                Some(std::mem::transmute(error_handler as *const ()));
            alsa_sys::snd_lib_error_set_handler(handler);
        }
    }
}

async fn start_mpris(
    action_tx: UnboundedSender<Action>,
    mpris_rx: UnboundedReceiver<FromPlayerWorker>,
    playerstatus: Arc<RwLock<PlayerStatus>>,
) -> std::result::Result<(), tokio::task::JoinError> {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            tokio::task::spawn_local(async move {
                let player = AmptermMpris::new(action_tx, playerstatus);
                let server = Server::new("ch.skew.ampterm", player)
                    .await
                    .expect("Failed to start MPRIS server!");
                server.imp().run(mpris_rx, &server).await;
            })
            .await
        })
        .await
}

async fn start_with_mpris(
    mut app: App,
    mut set: JoinSet<Result<()>>,
    action_tx: UnboundedSender<Action>,
    mpris_rx: UnboundedReceiver<FromPlayerWorker>,
    playerstatus: Arc<RwLock<PlayerStatus>>,
) -> Result<()> {
    let mpris = start_mpris(action_tx, mpris_rx, playerstatus);

    select! {
        _ = mpris => {
            panic!("MPRIS server has terminated before the player!");
        }
        res = app.run() => {
            match res {
                Ok(()) => Ok(()),
                Err(e) => panic!("UI panicked! Error: {}", e),
            }
        }
        res = set.join_next() => {
            match res {
                Some(report) => match report {
                    Ok(report) => match report {
                        Ok(_) => Ok(()),
                        Err(e) => panic!("A thread crashed: {}", e),
                    },
                    Err(_) => panic!("Failed to wait for the thread to run."),
                },
                None => unreachable!("No tasks completed??"),
            }
        }
    }
}

pub fn start_workers(
    action_tx: UnboundedSender<Action>,
    action_rx: UnboundedReceiver<Action>,
    mpris_tx: UnboundedSender<FromPlayerWorker>,
    config: Config,
    playerstatus: Arc<RwLock<PlayerStatus>>,
    tick_rate: f64,
    frame_rate: f64,
    #[cfg(test)] debug_tx: UnboundedSender<bool>,
) -> Result<(App, JoinSet<Result<()>>)> {
    let mut set = JoinSet::new();
    let mut pw = PlayerWorker::new(playerstatus, action_tx.clone(), config.clone());
    let mut qw = QueryWorker::new(action_tx.clone(), config.clone());
    let player_tx = pw.get_tx();
    let query_tx = qw.get_tx();

    #[cfg(test)]
    let app = App::new(
        config, action_tx, action_rx, mpris_tx, query_tx, player_tx, tick_rate, frame_rate,
        debug_tx,
    )?;

    #[cfg(not(test))]
    let app = App::new(
        config, action_tx, action_rx, mpris_tx, query_tx, player_tx, tick_rate, frame_rate,
    )?;

    // Start query worker
    set.spawn(async move { qw.run().await });
    // Start query worker
    set.spawn(async move { pw.run().await });
    Ok((app, set))
}

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    log_alsa_error();

    let args = Cli::parse();
    if let Some(msg) = args.is_valid() {
        return Err(eyre!(msg));
    }

    let data_str = if args.no_data {
        PathType::None
    } else if let Some(p) = args.data {
        PathType::Custom(p)
    } else {
        PathType::Default
    };

    let config_str = if args.no_config {
        PathType::None
    } else if let Some(p) = args.config {
        PathType::Custom(p)
    } else {
        PathType::Default
    };

    let config = Config::new(PathConfig::new(data_str, config_str))?;

    let playerstatus = Arc::from(RwLock::from(PlayerStatus::default()));
    let (action_tx, action_rx) = unbounded_channel::<Action>();
    let (mpris_tx, mpris_rx) = unbounded_channel::<FromPlayerWorker>();

    #[cfg(test)]
    panic!("Main invoked in test mode.");

    #[cfg(not(test))]
    {
        let (app, set) = start_workers(
            action_tx.clone(),
            action_rx,
            mpris_tx,
            config,
            playerstatus.clone(),
            args.tick_rate,
            args.frame_rate,
        )?;

        start_with_mpris(app, set, action_tx, mpris_rx, playerstatus).await
    }
}
