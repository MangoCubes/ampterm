use std::sync::Arc;

use clap::Parser;
use cli::Cli;
use color_eyre::{
    eyre::{eyre, Error},
    Result,
};
use mpris_server::Server;
use rodio::cpal::{self, traits::HostTrait};
use tokio::{
    select,
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        RwLock,
    },
    task::JoinSet,
};
use tracing::warn;

use crate::{
    action::action::Action,
    app::App,
    config::{pathconfig::PathConfig, Config},
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

pub fn run(
    args: Cli,
) -> Result<(
    UnboundedSender<Action>,
    Arc<RwLock<PlayerStatus>>,
    UnboundedReceiver<FromPlayerWorker>,
    JoinSet<Result<(), Error>>,
)> {
    let config = Config::new(PathConfig::new(
        args.data,
        args.no_data,
        args.config,
        args.no_config,
    ))?;

    let playerstatus = Arc::from(RwLock::from(PlayerStatus::default()));
    let (action_tx, action_rx) = mpsc::unbounded_channel();
    let (mpris_tx, mpris_rx) = mpsc::unbounded_channel();
    let mut qw = QueryWorker::new(action_tx.clone(), config.clone());
    let query_tx = qw.get_tx();

    // Set up audio stuff
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let handle = rodio::OutputStreamBuilder::from_device(device)
        .unwrap()
        .with_buffer_size(cpal::BufferSize::Fixed(4096))
        .open_stream()
        .unwrap();

    let mut set = JoinSet::new();

    let mut pw = PlayerWorker::new(
        playerstatus.clone(),
        handle,
        action_tx.clone(),
        config.clone(),
    );
    let player_tx = pw.get_tx();

    let mut app = App::new(
        config,
        action_tx.clone(),
        action_rx,
        mpris_tx,
        query_tx,
        player_tx,
        args.tick_rate,
        args.frame_rate,
    )?;

    // Start query worker
    set.spawn(async move { qw.run().await });
    // Start query worker
    set.spawn(async move { pw.run().await });
    // Start app
    set.spawn(async move { app.run().await });

    Ok((action_tx, playerstatus, mpris_rx, set))
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

    let Ok((action_tx, playerstatus, mpris_rx, mut set)) = run(args) else {
        return Ok(());
    };

    let local = tokio::task::LocalSet::new();
    let mpris = local.run_until(async {
        tokio::task::spawn_local(async move {
            let player = AmptermMpris::new(action_tx, playerstatus);
            let server = Server::new("ch.skew.ampterm", player)
                .await
                .expect("Failed to start MPRIS server!");
            server.imp().run(mpris_rx, &server).await;
        })
        .await
    });
    select! {
        _ = mpris => {
            panic!("MPRIS server has terminated before the player!");
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
