#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod image_buffer;
mod ui;

use clap::Parser;
use log::*;

/// Simple image viewer
#[derive(Debug, Parser)]
struct CliArgs {
    /// Use one of those levels: Trace, Debug, Info, Warn, Error
    #[arg(value_enum, short, long, default_value = "Warn")]
    log_level: Level,
    /// How many images should be cached?
    #[arg(short, long, default_value_t = 3)]
    cached_images: usize,
}

fn main() {
    let args = CliArgs::parse();
    stderrlog::new()
        .module(module_path!())
        .verbosity(args.log_level)
        .init()
        .unwrap();
    info!("Program arguments processed");

    ui::run(args);
}
