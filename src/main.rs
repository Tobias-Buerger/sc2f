#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod ui;
mod image_buffer;

use clap::Parser;
use log::*;
use stderrlog;

/// Simple image viewer
#[derive(Debug, Parser)]
struct CliArgs {
    /// Use one of those levels: Trace, Debug, Info, Warn, Error
    #[arg(value_enum, short, long, default_value = "Warn")]
    log_level: Level,
}

fn main() {
    let args = CliArgs::parse();
    stderrlog::new()
        .module(module_path!())
        .verbosity(args.log_level)
        .init()
        .unwrap();
    info!("Program arguments processed");

    ui::run(&args);
}
