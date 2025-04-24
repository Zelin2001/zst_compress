use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_compress_extract() {
    let original_dir = std::env::current_dir().unwrap();
    let test_dir = PathBuf::from("test");
    fs::create_dir_all(&test_dir).unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    // Run default test
    let result1 = std::panic::catch_unwind(|| {
        run_test_default().unwrap();
    });

    // Run directory test
    let result2 = std::panic::catch_unwind(|| {
        run_test_dir().unwrap();
    });

    // Clean up test directory regardless of test outcome
    std::env::set_current_dir(&original_dir).unwrap();
    let _ = fs::remove_dir_all("test");

    // Propagate any test failure
    if let Err(e) = result1 {
        std::panic::resume_unwind(e);
    }
    if let Err(e) = result2 {
        std::panic::resume_unwind(e);
    }
}

fn run_test_default() -> Result<(), Box<dyn std::error::Error>> {
    // Change to test directory
    let input = PathBuf::from("large_test.bin");
    let output = PathBuf::from("large_test.bin.tar.zst");

    // Generate 1MB of compressible data (repeating pattern)
    let pattern = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut data = Vec::with_capacity(1_000_000);
    while data.len() < 1_000_000 {
        data.extend_from_slice(pattern);
    }
    data.truncate(1_000_000);
    std::fs::write(&input, &data)?;

    // test preserved flag
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("-i")
        .arg(input.to_str().unwrap())
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Compress:"));
    assert!(input.exists());
    assert!(output.exists());

    // Verify compressed file is smaller
    let input_size = std::fs::metadata(&input)?.len();
    let output_size = std::fs::metadata(&output)?.len();
    assert!(
        output_size < input_size,
        "Compressed file should be smaller"
    );

    // Clean up for next test
    std::fs::remove_file(&output)?;

    // test normal flag
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("-i")
        .arg(input.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Compress:"));
    assert!(!input.exists());
    assert!(output.exists());

    // Then extract
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("-x")
        .assert()
        .success()
        .stdout(predicate::str::contains("Extract:"));
    assert!(input.exists());
    assert!(!output.exists());

    Ok(())
}

fn run_test_dir() -> Result<(), Box<dyn std::error::Error>> {
    // Create mixed test files
    fs::create_dir_all(PathBuf::from("dir"))?;
    let bin_file = PathBuf::from("dir/data.bin");
    let text_file = PathBuf::from("dir/text.txt");
    let output = PathBuf::from("dir.tar.zst");

    // Generate 1MB binary data
    let pattern = b"BINARYDATAPATTERN1234567890";
    let mut bin_data = Vec::with_capacity(1_000_000);
    while bin_data.len() < 1_000_000 {
        bin_data.extend_from_slice(pattern);
    }
    bin_data.truncate(1_000_000);
    std::fs::write(&bin_file, &bin_data)?;

    // Create text file
    std::fs::write(
        &text_file,
        "This is a test text file\nwith multiple lines\n",
    )?;

    // Run compression testing preserved flag
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::contains("Compress:"));
    assert!(bin_file.exists());
    assert!(text_file.exists());
    assert!(output.exists());

    // Verify compressed file is smaller
    let input_size = std::fs::metadata(&bin_file)?.len();
    let output_size = std::fs::metadata(&output)?.len();
    assert!(
        output_size < input_size,
        "Compressed file should be smaller"
    );

    // Clean up for next test
    std::fs::remove_file(&output)?;

    // test normal flag
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Compress:"));
    assert!(!bin_file.exists());
    assert!(!text_file.exists());
    assert!(output.exists());

    // Then extract
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("-x")
        .assert()
        .success()
        .stdout(predicate::str::contains("Extract:"));
    assert!(bin_file.exists());
    assert!(text_file.exists());
    assert!(!output.exists());

    Ok(())
}
