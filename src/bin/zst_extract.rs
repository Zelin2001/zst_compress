use std::env::current_dir;
use std::fs::{read_dir, remove_file};
use std::process::Command;

fn main() {
    // Set the selected patterns
    let str_archive = ".tar.zst";
    let str_archive_list = "_filelist.txt";
    let str_tools = "zst_";

    // Walk through videos
    match read_dir(current_dir().unwrap()) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        match entry.file_name().to_str() {
                            Some(f_name) => {
                                // Select files
                                if (f_name.len() >= str_archive.len()
                                    && f_name.rfind(str_archive)
                                        == Some(f_name.len() - str_archive.len()))
                                    && f_name.find(str_tools) != Some(0)
                                {
                                    // Decompress
                                    print!("\nExtracting: {}", f_name);
                                    let f_ori: &str = &f_name[0..f_name.rfind(str_archive).unwrap()];
                                    let mut do_compress = Command::new("tar")
                                        .arg("-xf")
                                        .arg(f_name)
                                        .spawn()
                                        .expect(&format!("Failed to compress {}", f_name));
                                    let _ = do_compress.wait();
                                    print!(" -> {}", f_ori);

                                    // Remove original file
                                    match remove_file(f_name) {
                                        Ok(_) => (),
                                        Err(e) => {
                                            eprintln!(
                                                "Error, couldn't remove original {}: {}",
                                                f_name, e
                                            );
                                        }
                                    }
                                    let f_list: &str = &format!("{}{}", f_ori, str_archive_list);
                                    match remove_file(f_list) {
                                        Ok(_) => (),
                                        Err(e) => {
                                            eprintln!(
                                                "Error, couldn't remove original {}: {}",
                                                f_list, e
                                            );
                                        }
                                    }
                                }
                            }
                            None => eprintln!("Error reading {:?}", entry.file_name()),
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
