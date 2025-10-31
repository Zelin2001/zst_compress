use crate::auxiliary::DirGuard;
use crate::exec::{RET_DIR_ERROR, RET_ITEM_ERROR, entry_archive};
use clap::{ArgAction, Parser};
use glob::Pattern;
use regex::Regex;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Args for CLI use
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Directory to start processing
    #[arg(value_name = "DIRECTORY")]
    pub directory_start: PathBuf,

    /// Preview what would be done without executing
    #[arg(short = 'n', long)]
    pub dryrun: bool,

    /// Exclude files matching glob pattern(s)
    #[arg(short, long, value_name = "PATTERN", action = ArgAction::Append)]
    pub exclude: Option<String>,

    /// Exclude files matching regex pattern(s)
    #[arg(long, value_name = "PATTERN", action = ArgAction::Append)]
    pub excludere: Option<String>,

    /// Extract files (decompress mode)
    #[arg(short = 'x', long)]
    pub extract: bool,

    /// Create compression message file on the compression
    #[arg(short, long)]
    pub flag: bool,

    /// Include files matching glob pattern(s)
    /// [default: *]
    #[arg(short, long, value_name = "PATTERN", action = ArgAction::Append)]
    pub include: Option<String>,

    /// Include files matching regex pattern(s)
    #[arg(long, value_name = "PATTERN", action = ArgAction::Append)]
    pub includere: Option<String>,

    /// Directory listing depth for logs
    /// in *_archive_filelist.txt [default: 4]
    #[arg(short, long, value_name = "LEVEL")]
    pub leveldir: Option<u8>,

    /// Keep original (do not delete) files after compression
    #[arg(short, long)]
    pub preserve: bool,

    /// Suppress output except errors (NO FUNCTION)
    #[arg(short, long)]
    pub quiet: bool,

    /// Output directory [default: DIRECTORY to start]
    #[arg(short, long, value_name = "DIRECTORY")]
    pub target: Option<PathBuf>,

    /// Show detailed progress information (NO FUNCTION)
    #[arg(short, long)]
    pub verbose: bool,

    /// Zstandard compress level, 1(fastest) to 22(smallest);
    /// [default: 5]
    #[arg(short, long, value_name = "LEVEL")]
    pub zstdlevel: Option<i32>,
}
/// Do the cli parsing
pub fn cli() -> ExitCode {
    let args = Args::parse();
    match batch_archive(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(ret) => ExitCode::from(ret),
    }
}

/// Compress or decompress all items in a folder
pub fn batch_archive(args: Args) -> Result<(), u8> {
    let mut ret = 0;
    let level_tree = args.leveldir.unwrap_or(4);
    let compress = !args.extract;

    let start_dir = &args.directory_start;
    let _guard = DirGuard::new(start_dir)?;
    let target_dir = if let Some(target) = &args.target {
        Path::new(target)
    } else {
        start_dir.as_path()
    };
    // Walk through videos
    match read_dir(start_dir) {
        Ok(entries) => {
            let entries: Vec<_> = entries.collect();
            let mut valid_entries: Vec<_> = vec![];
            for entry_result in entries.into_iter() {
                match entry_result {
                    Ok(entry) => {
                        let file_path = entry.path();
                        if should_process_file(
                            &file_path,
                            &args.include,
                            &args.exclude,
                            &args.includere,
                            &args.excludere,
                        ) {
                            valid_entries.push(entry.path());
                        }
                    }
                    Err(e) => {
                        eprintln!("出错了! Error: {e}");
                        return Err(RET_DIR_ERROR);
                    }
                }
            }
            let total_items = valid_entries.len();
            if total_items < 1 {
                eprintln!("No item in {:?} to process.", &start_dir)
            }
            for (current_item, entry_path) in valid_entries.into_iter().enumerate() {
                if entry_archive(
                    entry_path.as_path(),
                    compress,
                    args.preserve,
                    args.flag,
                    target_dir,
                    level_tree,
                    args.zstdlevel.unwrap_or(5_i32),
                    current_item + 1,
                    total_items,
                    args.dryrun,
                ) != Ok(())
                {
                    ret = RET_ITEM_ERROR
                }
            }
        }
        Err(e) => eprintln!("出错了! Error: {e}"),
    };

    match ret {
        0 => Ok(()),
        _ => Err(ret),
    }
}

/// Check if a file path matches the include and exclude patterns
/// Supports both glob patterns and regular expressions
fn should_process_file(
    file_path: &Path,
    include_patterns: &Option<String>,
    exclude_patterns: &Option<String>,
    include_regex_patterns: &Option<String>,
    exclude_regex_patterns: &Option<String>,
) -> bool {
    let file_name = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    // Check exclude glob patterns first
    if let Some(exclude) = exclude_patterns
        && let Ok(pattern) = Pattern::new(exclude)
        && pattern.matches(file_name)
    {
        return false;
    }

    // Check exclude regex patterns
    if let Some(exclude_regex) = exclude_regex_patterns
        && let Ok(regex) = Regex::new(exclude_regex)
        && regex.is_match(file_name)
    {
        return false;
    }

    // Check include patterns - if any include pattern matches, process the file
    let mut should_include = false;

    // Check include glob patterns (default to "*" if None)
    if let Some(include) = include_patterns {
        if let Ok(pattern) = Pattern::new(include)
            && pattern.matches(file_name)
        {
            should_include = true;
        }
    } else {
        // Default behavior: include everything if no include pattern specified
        should_include = true;
    }

    // Check include regex patterns
    if let Some(include_regex) = include_regex_patterns
        && let Ok(regex) = Regex::new(include_regex)
        && regex.is_match(file_name)
    {
        should_include = true;
    }

    // If both include and include regex are None, default to "*"
    if include_patterns.is_none() && include_regex_patterns.is_none() {
        should_include = true;
    }

    should_include
}
