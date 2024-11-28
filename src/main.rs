use std::env::{current_dir, current_exe, set_var, var};
use std::fs::{read_dir, remove_dir_all, remove_file, File};
use std::io::prelude::*;
use std::process::{Command, Stdio};

fn main() {
    // Add `./zst_bin/` to $PATH
    let mut path_bin = current_exe().unwrap().parent().unwrap().to_owned();
    path_bin.push("zst_bin");
    let path_all = path_bin.to_str().unwrap().to_string() + ":" + &(var("PATH").unwrap());
    set_var("PATH", path_all);

    // Set the skipped patterns
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
                                // Skip files
                                if (f_name.len() < str_archive.len()
                                    || f_name.rfind(str_archive)
                                        != Some(f_name.len() - str_archive.len()))
                                    && (f_name.len() < str_archive_list.len()
                                        || f_name.rfind(str_archive_list)
                                            != Some(f_name.len() - str_archive_list.len()))
                                    && f_name.find(str_tools) != Some(0)
                                {
                                    // Make filelist
                                    let do_list = Command::new("eza.exe")
                                        .arg("-lT")
                                        .arg("-L4")
                                        .arg(f_name)
                                        .stdout(Stdio::piped())
                                        .spawn()
                                        .expect(&format!("Failed to list files from {}", f_name));

                                    let mut f_list =
                                        File::create(format!("{}{}", f_name, str_archive_list))
                                            .expect(&format!("Failed to create file: {}", f_name));
                                    let mut buf = vec![];
                                    match do_list
                                        .stdout
                                        .expect("Failed to open stdout")
                                        .read_to_end(&mut buf)
                                    {
                                        Err(e) => {
                                            eprintln!("Error, couldn't read stdout: {}", e);
                                        }
                                        Ok(_) => {
                                            let _ = f_list.write_all(&buf);
                                        }
                                    }

                                    // Compress
                                    print!("\nCompressing: {}", f_name);
                                    let f_out = &format!("{}{}", f_name, str_archive);
                                    let mut do_compress = Command::new("tar")
                                        .arg("-cf")
                                        .arg(f_out)
                                        .arg(f_name)
                                        .spawn()
                                        .expect(&format!("Failed to compress {}", f_name));
                                    let _ = do_compress.wait();
                                    print!(" -> {}", f_out);

                                    // Remove original file
                                    if entry.file_type().unwrap().is_dir() {
                                        match remove_dir_all(f_name) {
                                            Ok(_) => (),
                                            Err(e) => {
                                                eprintln!(
                                                    "Error, couldn't remove original {}: {}",
                                                    f_name, e
                                                );
                                            }
                                        }
                                    } else {
                                        match remove_file(f_name) {
                                            Ok(_) => (),
                                            Err(e) => {
                                                eprintln!(
                                                    "Error, couldn't remove original {}: {}",
                                                    f_name, e
                                                );
                                            }
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
