use crate::batch::batch_archive;
use clap::Parser;
use std::env::current_dir;
use std::process::ExitCode;

/// Args for CLI use
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Preseve original files after compression
    #[arg(short, long)]
    pub preserve: bool,

    /// Leave flag text pointing to compression target
    #[arg(short, long)]
    pub flag: bool,

    /// Select a single input file instead of ./*
    #[arg(short, long)]
    pub input: Option<String>,

    /// Extract file from batch archived
    #[arg(short)]
    pub x: bool,

    /// Select recursive level for listing directory,
    /// default to 4
    #[arg(short, long)]
    pub level: Option<u8>,

    /// Target location for oprated files, default to current
    #[arg(short, long)]
    pub target: Option<String>,
    // /// Specific files to operate on
    // #[arg(short, long)]
    // pub files: Option<Vec<String>>,
}

/// Do the cli parsing
pub fn cli(mut compress: bool) -> ExitCode {
    let args = Args::parse();
    if args.x.clone() {
        compress = false;
    }

    match batch_archive(
        current_dir().expect("Fatal: No current working directory, quit."),
        args,
        compress,
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(ret) => ExitCode::from(ret),
    }
}
