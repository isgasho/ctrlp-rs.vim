#[macro_use]
extern crate log;
extern crate neovim_lib;
extern crate simplelog;

pub mod handler;
pub mod event;

use handler::NeovimHandler;
use event::Event;

use std::error::Error;
use std::sync::mpsc;

use neovim_lib::neovim::Neovim;
use neovim_lib::neovim_api::NeovimApi;
use neovim_lib::session::Session;

use simplelog::{Config, LogLevel, LogLevelFilter, WriteLogger};


fn main() {
    use std::process;

    init_logging().expect("scorched earth: unable to initialize logger.");

    match start_program() {
        Ok(_) => process::exit(0),

        Err(msg) => {
            error!("{}", msg);
            process::exit(1);
        }
    };
}


fn init_logging() -> Result<(), Box<Error>> {
    use std::env;
    use std::env::VarError;
    use std::fs::File;

    let _log_level_filter =
        match env::var("LOG_LEVEL").unwrap_or(String::from("trace")).to_lowercase().as_ref() {
            "debug" => LogLevelFilter::Debug,
            "error" => LogLevelFilter::Error,
            "info" => LogLevelFilter::Info,
            "off" => LogLevelFilter::Off,
            "trace" => LogLevelFilter::Trace,
            "warn" => LogLevelFilter::Warn,
            _ => LogLevelFilter::Off,
        };

    let log_level_filter = LogLevelFilter::Info;

    let config = Config {
        time: Some(LogLevel::Error),
        level: Some(LogLevel::Error),
        target: Some(LogLevel::Error),
        location: Some(LogLevel::Error),
    };

    let filepath = match env::var("LOG_FILE") {
        Err(err) => {
            match err {
                VarError::NotPresent => return Ok(()),
                e @ VarError::NotUnicode(_) => {
                    return Err(Box::new(e));
                }
            }
        }
        Ok(path) => path.to_owned(),
    };

    let log_file = File::create(filepath)?;

    WriteLogger::init(log_level_filter, config, log_file).unwrap();

    Ok(())
}


fn start_program() -> Result<(), Box<Error>> {
    info!("Connection to neovim");
    let (sender, receiver) = mpsc::channel();
    let mut session = Session::new_parent()?;
    session.start_event_loop_handler(NeovimHandler(sender));
    let mut nvim = Neovim::new(session);


    info!("Subscribing to events");
    nvim.subscribe("startup").expect("Failed to subscribe to event: startup");
    nvim.subscribe("shutdown").expect("Failed to subscribe to event: shutdown");

    start_event_loop(receiver, nvim);

    Ok(())
}


fn start_event_loop(receiver: mpsc::Receiver<Event>, mut nvim: Neovim) {
    info!("Starting event loop");
    loop {
        match receiver.recv() {
            Ok(Event::Startup) => {
                info!("Starting up");
                nvim.command("echom \"ctrlp startup finished\"").unwrap();
            }
            Ok(Event::Shutdown) => break,
            _ => {
                info!("Shuting down");
            }
        }
    }
}
