mod ui;

use clap::Parser;
use simple_logger;
use log::*;

/// Simple image viewer
#[derive(Debug, Parser)]
struct CliArgs {
    /// Use one of those levels: Trace, Debug, Info, Warn, Error
    #[arg(value_enum, short, long, default_value = "Warn")]
    log_level: Level,
}

fn main() {
    let args = CliArgs::parse();
    simple_logger::init_with_level(args.log_level).unwrap();
    info!("Program arguments processed");

    ui::run(&args);
}
