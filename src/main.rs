use std::process::{Command, Stdio};
use clap::Parser;
use simplelog::{ConfigBuilder, LevelFilter, SimpleLogger};
use log::{info, error};
use std::str;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    remote: String,

    #[arg(short, long)]
    branch: String,

    #[arg(long)]
    build_cmd: String,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    let args = Args::parse();
    let log_level = match args.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    let log_cfg = ConfigBuilder::new().build();
    SimpleLogger::init(log_level, log_cfg).unwrap();

    // Fetch from remote
    let fetch = Command::new("git")
        .arg("fetch")
        .arg(&args.remote)
        .arg(&args.branch)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("git fetch failed");

    if !fetch.success() {
        error!("Failed to fetch from remote.");
        std::process::exit(1);
    }

    // Get the local hash
    let local_hash = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("git rev-parse failed on HEAD");

    let local_hash_str = str::from_utf8(&local_hash.stdout)
        .expect("Failed to parse local hash");

    // Get the remote hash
    let remote_hash = Command::new("git")
        .arg("rev-parse")
        .arg(format!("{}/{}", args.remote, args.branch))
        .output()
        .expect("git rev-parse failed on remote branch");

    let remote_hash_str = str::from_utf8(&remote_hash.stdout)
        .expect("Failed to parse remote hash");

    // Compare the hashes
    if local_hash_str.trim() == remote_hash_str.trim() {
        info!("No changes to build");
        return;
    }

    info!("Running build command: {}", &args.build_cmd);

    let build = Command::new("sh")
        .arg("-c")
        .arg(&args.build_cmd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to run build command");

    if !build.success() {
        error!("Build failed");
        std::process::exit(1);
    }
}

