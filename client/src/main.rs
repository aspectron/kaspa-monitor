#![warn(clippy::all, rust_2018_idioms)]
// hide console window on Windows in release mode
#![cfg_attr(
    all(not(debug_assertions), not(feature = "console")),
    windows_subsystem = "windows"
)]

pub mod core;
pub mod error;
pub mod events;
pub mod imports;
pub mod modules;
pub mod node;
pub mod result;
pub mod services;

use crate::imports::*;

use workflow_egui as iris;

fn main() {
    let options = iris::frame::options::Options::<Core>::new(
        "Kaspa Node Monitor".to_string(),
        "kaspa-node-monitor".to_string(),
    );

    iris::frame::main(options, None, |cc, runtime| {
        log_info!("--- creating applcation ---");

        Ok(Box::new(Core::try_new(cc, runtime)?))
    });
}
