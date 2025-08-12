mod generate;
mod progress;
mod spec;
mod utils;

use clap::{Arg, Command};
use shared::logs::setup_logger;
use spec::VersionsSpec;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::runtime::Runtime;

fn parse_path(v: &str) -> anyhow::Result<PathBuf> {
    let path = PathBuf::from(v);
    if path.exists() {
        Ok(path)
    } else {
        Err(anyhow::Error::msg("The specified file does not exist"))
    }
}

const LOGS_FILENAME: &str = "builder.log";

pub fn get_logs_path(logs_dir: &Path) -> PathBuf {
    if !logs_dir.exists() {
        std::fs::create_dir_all(logs_dir).expect("Failed to create logs directory");
    }
    logs_dir.join(LOGS_FILENAME)
}

fn main() -> anyhow::Result<()> {
    unsafe {
        std::env::set_var("RUST_LIB_BACKTRACE", "1");
    }

    let matches = Command::new("generate-instance")
        .about("Generates instances based on a specification file")
        .arg(
            Arg::new("spec_file")
                .help("Path to the specification file")
                .required(true)
                .short('s')
                .value_parser(parse_path),
        )
        .arg(
            Arg::new("output_dir")
                .help("Output directory")
                .default_value("./generated"),
        )
        .arg(
            Arg::new("work_dir")
                .help("Working directory")
                .default_value("./workdir"),
        )
        .arg(
            Arg::new("delete_remote_instances")
                .help("Comma-separated remote instance names to delete from fetched manifest")
                .long("delete-remote")
                .num_args(1..)
                .use_value_delimiter(true)
                .value_delimiter(',')
                .value_name("NAME"),
        )
        .get_matches();

    let spec_file = matches.get_one::<PathBuf>("spec_file").unwrap();
    let output_dir = matches.get_one::<String>("output_dir").unwrap();
    let output_dir = PathBuf::from(output_dir);
    let work_dir = matches.get_one::<String>("work_dir").unwrap();
    let work_dir = PathBuf::from(work_dir);

    let spec_file_path = spec_file.clone();
    let output_dir_path = output_dir.clone();
    let work_dir_path = work_dir.clone();

    setup_logger(&get_logs_path(&work_dir));

    let rt = Runtime::new().unwrap();
    let spec = rt.block_on(VersionsSpec::from_file(&spec_file_path))?;
    let delete_remote_set: Option<HashSet<String>> = matches
        .get_many::<String>("delete_remote_instances")
        .map(|vals| vals.map(|s| s.to_string()).collect());

    rt.block_on(spec.generate(&output_dir_path, &work_dir_path, delete_remote_set.as_ref()))
}
