use assert_cmd::Command;
use predicates::prelude::*;
use std::env::set_current_dir;
use std::fs::{create_dir_all, metadata, remove_dir_all, write};
use std::path::PathBuf;

#[test]
fn test_cli() {
    // Run tests
    run_test(
        "tests/data_default",
        &[],
        "/2) Compress:",
        &vec![false, false, false, true, true, true],
        &["-x"],
        "/3) Extract:",
        &vec![true, true, true, false, false, false],
    )
    .unwrap();
    run_test(
        "tests/data_preserve",
        &["-p"],
        "2) Compress:",
        &vec![true, true, true, true, true, true],
        &["-p", "-x"],
        "/5) Extract:",
        &vec![true, true, true, true, true, true],
    )
    .unwrap();
    run_test(
        "tests/data_single",
        &["-i", "large_test.bin"],
        "(1/1) Compress:",
        &vec![true, true, false, false, false, true],
        &["-p", "-x"],
        "/2) Extract:",
        &vec![true, true, true, false, false, true],
    )
    .unwrap();
}

/// Runs a complete test cycle with compression and extraction
///
/// # Arguments
/// * `test_data_dir` - Directory path for test files
/// * `compress_args` - CLI args for compression (e.g. ["-p"])
/// * `compress_expect` - Expected stdout text during compression
/// * `compress_files_status` - Expected file states after compression:
///   [dir/data1.bin, dir/text.txt, large_test.bin, dir.tar.zst,
///    dir_archived-filelist.txt, large_test.bin.tar.zst]
/// * `decompress_args` - CLI args for extraction (e.g. ["-x"])
/// * `decompress_expect` - Expected stdout text during extraction
/// * `decompress_files_status` - Expected file states after extraction:
///   [dir/data1.bin, dir/text.txt, large_test.bin, dir.tar.zst,
///    dir_archived-filelist.txt, large_test.bin.tar.zst]
fn run_test(
    test_data_dir: &str,
    compress_args: &[&str],
    compress_expect: &str,
    compress_files_status: &Vec<bool>,
    decompress_args: &[&str],
    decompress_expect: &str,
    decompress_files_status: &Vec<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize test
    let test_dir = PathBuf::from(test_data_dir);
    let original_dir = run_setup(&test_dir)?;

    // Create files
    let (filenames, filesizes) = run_test_files_create()?;

    // Test compression
    run_test_command(compress_args, compress_expect)?;
    run_test_files_check(&filenames, &filesizes, compress_files_status)?;

    // Test extraction
    run_test_command(decompress_args, decompress_expect)?;
    run_test_files_check(&filenames, &filesizes, decompress_files_status)?;

    // Clean up test
    run_cleanup(&original_dir, &test_dir)?;

    Ok(())
}

/// Sets up test environment by creating directory and changing working directory
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// let test_dir = PathBuf::from("tests/doc_test_setup");
/// let original_dir = run_setup(&test_dir).unwrap();
/// assert!(test_dir.exists());
/// assert_eq!(std::env::current_dir().unwrap(), test_dir);
/// run_cleanup(&original_dir, &test_dir).unwrap();
/// ```
fn run_setup(test_dir: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Save the directory
    let original_dir = std::env::current_dir()?;
    create_dir_all(test_dir)?;
    set_current_dir(test_dir)?;

    return Ok(original_dir);
}

/// Cleans up test environment by restoring original directory and removing test files
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// let test_dir = PathBuf::from("tests/doc_test_cleanup");
/// let original_dir = run_setup(&test_dir).unwrap();
/// run_cleanup(&original_dir, &test_dir).unwrap();
/// assert_eq!(std::env::current_dir().unwrap(), original_dir);
/// assert!(!test_dir.exists());
/// ```
fn run_cleanup(
    original_dir: &PathBuf,
    test_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Clean up test directory regardless of test outcome
    std::env::set_current_dir(original_dir)?;
    let _ = remove_dir_all(test_dir);

    return Ok(());
}

/// Creates test files with specific patterns for compression testing
///
/// Creates:
/// - 1MB binary file (dir/data1.bin)
/// - Text file (dir/text.txt)
/// - 2MB compressible data file (large_test.bin)
///
/// Returns tuple of:
/// - Vector of all test file paths
/// - Vector of original file sizes for first 3 files
fn run_test_files_create() -> Result<(Vec<PathBuf>, Vec<u64>), Box<dyn std::error::Error>> {
    // Create mixed test files
    create_dir_all(PathBuf::from("dir"))?;
    let dir_bin_input = PathBuf::from("dir/data1.bin");
    let dir_text_input = PathBuf::from("dir/text.txt");
    let file_input = PathBuf::from("large_test.bin");
    let dir_output = PathBuf::from("dir.tar.zst");
    let dir_filelist_output = PathBuf::from("dir_archived-filelist.txt");
    let file_output = PathBuf::from("large_test.bin.tar.zst");

    // Generate 1MB binary data
    let pattern = b"BINARYDATAPATTERN1234567890";
    let mut dir_bin_data = Vec::with_capacity(1_000_000);
    while dir_bin_data.len() < 1_000_000 {
        dir_bin_data.extend_from_slice(pattern);
    }
    dir_bin_data.truncate(1_000_000);
    write(&dir_bin_input, &dir_bin_data)?;

    // Create text file
    write(
        &dir_text_input,
        "This is a test text file\nwith multiple lines\n",
    )?;

    // Generate 2MB of compressible data (repeating pattern)
    let pattern = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut data = Vec::with_capacity(1_000_000);
    while data.len() < 2_000_000 {
        data.extend_from_slice(pattern);
    }
    data.truncate(2_000_000);
    write(&file_input, &data)?;

    return Ok((
        vec![
            dir_bin_input.clone(),
            dir_text_input.clone(),
            file_input.clone(),
            dir_output,
            dir_filelist_output,
            file_output,
        ],
        vec![
            metadata(&dir_bin_input)?.len(),
            metadata(&dir_text_input)?.len(),
            metadata(&file_input)?.len(),
        ],
    ));
}

/// Verifies file states match expected status after compression/extraction
///
/// # Arguments
/// * `filenames` - All test file paths
/// * `filesizes` - Original sizes of input files
/// * `status` - Expected existence state for each file
///
/// Checks:
/// - Files exist/not exist per status flags
/// - Compressed files are smaller than originals
fn run_test_files_check(
    filenames: &Vec<PathBuf>,
    filesizes: &Vec<u64>,
    status: &Vec<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    for file_index in 0..filenames.len() {
        match status[file_index] {
            true => {
                // Verify the file exists with better error message
                let path = &filenames[file_index];
                assert!(
                    path.exists(),
                    "File {} should exist but doesn't",
                    path.display()
                );

                // Verify compressed file is smaller with better error message
                if file_index == 3 || file_index == 5 {
                    let input_size = filesizes[file_index - 3];
                    let output_size = metadata(path)?.len();
                    assert!(
                        output_size < input_size,
                        "Compressed file {} ({} bytes) should be smaller than input ({} bytes)",
                        path.display(),
                        output_size,
                        input_size
                    );
                }
                // Verify filelist contents if it exists
                else if file_index == 4 && path.exists() {
                    let contents = std::fs::read_to_string(path)?;
                    println!("{}", contents);
                    assert!(
                        contents.contains("data1.bin"),
                        "Filelist should contain 'data1.bin'"
                    );
                    assert!(
                        contents.contains("text.txt"), 
                        "Filelist should contain 'text.txt'"
                    );
                }
            }
            false => {
                // Verify the file is removed
                assert!(!filenames[file_index].exists());
            }
        }
    }

    return Ok(());
}

/// Executes CLI command and verifies output
///
/// # Arguments
/// * `args` - Command line arguments to pass
/// * `expected_output` - Text that should appear in stdout
///
/// Uses assert_cmd to run the binary and verify:
/// - Exit code is success
/// - Output contains expected text
fn run_test_command(
    args: &[&str],
    expected_output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    for arg in args {
        cmd.arg(arg);
    }
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(expected_output));
    Ok(())
}
