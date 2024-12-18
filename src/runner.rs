use crate::batch::batch_archive;
use clap::Parser;
use std::process::ExitCode;

/// Args for CLI use
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Preseve original files, do not delete
    #[arg(short, long)]
    pub preserve: bool,

    /// Target location for oprated files
    #[arg(short, long)]
    pub target: Option<String>,

    // /// Specific files to operate on
    // #[arg(short, long)]
    // pub files: Option<Vec<String>>,
}

/// Do the cli parsing
pub fn cli(compress: bool) -> ExitCode {
    let args = Args::parse();

    match batch_archive(args, compress) {
        Ok(()) => ExitCode::SUCCESS,
        Err(ret) => ExitCode::from(ret),
    }
}
