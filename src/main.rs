use std::process::{Command, Stdio};
use shell_words::split;
use clap::Parser;
use simplelog::{ConfigBuilder, LevelFilter, SimpleLogger};
use log::{info, error};

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
        _ => LevelFilter::Trace
    };
    let log_cfg = ConfigBuilder::new().build();
    SimpleLogger::init(log_level, log_cfg).unwrap();

    let fetch = Command::new("git")
        .arg("fetch")
        .arg(&args.remote)
        .arg(&args.branch)
        .stdout(Stdio::piped())
        .output()
        .expect("git fetch failed");

    let local_hash = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("git rev-parse failed on HEAD");

    let remote_hash = Command::new("git")
        .arg("rev-parse")
        .arg(format!("{}/{}", args.remote, args.branch))
        .output()
        .expect("git rev-parse failed on remote branch");

    if (local_hash.stdout == remote_hash.stdout) && (fetch.status.success()) {
        info!("No changes to build");
        return;
    }
    let build_cmd = split(&args.build_cmd).unwrap();
    info!("Running build command: {}", build_cmd.join(" "));
    let build = Command::new(&build_cmd[0])
        .args(&build_cmd[1..])
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to run build command");
    if !build.status.success() {
        error!("Build failed");
        std::process::exit(1);
    }
}
