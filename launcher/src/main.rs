#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod auth;
mod config;
mod constants;
mod lang;
mod launcher;
mod update_app;
mod utils;
mod vendor;
mod version;

use clap::{Arg, ArgAction, Command};
use config::runtime_config::{get_logs_path, Config};
use utils::set_sigint_handler;

use shared::logs::setup_logger;

fn main() {
    set_sigint_handler();
    setup_logger(&get_logs_path());

    let matches = Command::new("generate-instance")
        .about("Generates instances based on a specification file")
        .arg(
            Arg::new("launch")
                .help("Launch the game in the last used configuration")
                .long("launch")
                .short('l')
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let config = Config::load();
    update_app::app::run_gui(&config);
    app::launcher_app::run_gui(config, matches.get_flag("launch"));
}
