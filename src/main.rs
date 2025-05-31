use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use tracing::warn;

use crate::app::App;

mod action;
mod app;
mod cli;
mod components;
mod config;
mod errors;
mod logging;
mod osclient;
mod playerworker;
mod queryworker;
mod stateful;
mod stateless;
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
    let mut app = App::new(args.tick_rate, args.frame_rate)?;
    app.run().await?;
    Ok(())
}
