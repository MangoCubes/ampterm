use clap::Parser;
use cli::Cli;
use color_eyre::{eyre::eyre, Result, Section};
use rodio::{
    cpal::{self, traits::HostTrait, SupportedBufferSize},
    DeviceTrait, SupportedStreamConfig,
};
use tokio::{sync::mpsc, task::JoinSet};
use tracing::warn;

use crate::{app::App, config::Config, playerworker::PlayerWorker, queryworker::QueryWorker};

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
mod osclient;
mod playerworker;
mod queryworker;
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
        unsafe {
            let handler: alsa_sys::snd_lib_error_handler_t =
                Some(std::mem::transmute(error_handler as *const ()));
            alsa_sys::snd_lib_error_set_handler(handler);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    log_alsa_error();

    let args = Cli::parse();
    let config = Config::new(args.data, args.config)?;

    // Set up audio stuff
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let default_config = device.default_output_config()?;
    let (_stream, handle) = rodio::OutputStream::try_from_device_config(
        &device,
        SupportedStreamConfig::new(
            default_config.channels(),
            default_config.sample_rate(),
            SupportedBufferSize::Range {
                min: 4096,
                max: 4096,
            },
            default_config.sample_format(),
        ),
    )
    .unwrap();

    let mut set = JoinSet::new();

    // Communication channel for [`App`]
    let (action_tx, action_rx) = mpsc::unbounded_channel();
    let mut qw = QueryWorker::new(action_tx.clone(), config.clone());
    let query_tx = qw.get_tx();
    // Start query worker
    set.spawn(async move { qw.run().await });

    let mut pw = PlayerWorker::new(handle, action_tx.clone(), config.clone());
    let player_tx = pw.get_tx();
    // Start query worker
    set.spawn(async move { pw.run().await });

    let mut app = App::new(
        config,
        action_tx,
        action_rx,
        query_tx,
        player_tx,
        args.tick_rate,
        args.frame_rate,
    )?;
    set.spawn(async move { app.run().await });

    while let Some(res) = set.join_next().await {
        match res {
            Ok(r) => match r {
                Ok(_) => {
                    return Ok(());
                }
                Err(r) => {
                    return Err(eyre!(r).with_note(|| "A thread has crashed."));
                }
            },
            Err(r) => {
                return Err(eyre!(r).with_note(|| "Failed to wait for the thread to run."));
            }
        }
    }
    Ok(())
}
