// TODO: Capture ^C
use std::env::{current_dir, current_exe, set_var, var};
use std::fs::{read_dir, remove_dir_all, remove_file, DirEntry, File};
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};

// Set the skipped / selected patterns
static S_ARCHIVE: &str = ".tar.zst";
static S_ARCHILIST: &str = "_filelist.txt";
static S_TOOL: &str = "zst_";
static S_BIN: &str = "zst_bin";

/// Compress or decompress all items in a folder
pub fn batch_archive(compress: bool) -> () {
    // Add `./zst_bin/` to $PATH
    if compress {
        let mut path_bin = current_exe().unwrap().parent().unwrap().to_owned();
        path_bin.push(S_BIN);
        let mut path_all = path_bin.to_str().unwrap().to_string();
        if cfg!(target_os = "windows") {
            path_all = path_all + ";";
        } else {
            path_all = path_all + ":";
        }
        path_all = path_all + &(var("PATH").unwrap());
        set_var("PATH", &path_all);
    }

    // Walk through videos
    match read_dir(current_dir().unwrap()) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => entry_archive(entry, compress),
                    Err(e) => eprintln!("出错了! Error: {}", e),
                }
            }
        }
        Err(e) => eprintln!("出错了! Error: {}", e),
    }
}

/// Compress or decompress 1 item
fn entry_archive(entry: DirEntry, compress: bool) -> () {
    match entry.file_name().to_str() {
        Some(f_name) => {
            // Check if is directory
            let f_is_dir = entry.file_type().unwrap().is_dir();

            // Selected archive files
            if (f_name.len() >= S_ARCHIVE.len()
                && f_name.rfind(S_ARCHIVE) == Some(f_name.len() - S_ARCHIVE.len()))
                && f_name.find(S_TOOL) != Some(0)
            {
                // Decompress and clean
                if !compress {
                    print!("Extracting: {}", f_name);
                    let f_ori: &str = &f_name[0..f_name.rfind(S_ARCHIVE).unwrap()];
                    let do_extract = Command::new("tar")
                        .arg("-xf")
                        .arg(f_name)
                        .spawn()
                        .expect(&format!("出错了! Failed to extract {}", f_name));
                    match do_extract.wait_with_output() {
                        Ok(out) => {
                            if out.status.code() != Some(0) {
                                eprintln!("出错了! tar returned: {:?}", out.status.code());
                                return;
                            }
                        }
                        Err(e) => {
                            eprintln!("出错了! Error with tar compression: {}", e);
                            return;
                        }
                    }
                    print!(" -> {}\n", f_ori);

                    // Remove original file
                    let _ = f_remove_print(f_name, false);
                    let f_list: &str = &format!("{}{}", f_ori, S_ARCHILIST);
                    if Path::exists(Path::new(f_list)) {
                        let _ = f_remove_print(f_list, false);
                    }
                }
            }
            // Skip filelists and tools
            else if f_name.find(S_TOOL) == Some(0)
                || (f_name.len() >= S_ARCHILIST.len()
                    && f_name.rfind(S_ARCHILIST) == Some(f_name.len() - S_ARCHILIST.len()))
            { // Do nothing
            }
            // Compress, mark the filelist and clean
            else {
                if compress {
                    // Make filelist
                    if entry.file_type().unwrap().is_dir() {
                        let do_list = Command::new("eza")
                            .arg("-lT")
                            .arg("-L4")
                            .arg(f_name)
                            .stdout(Stdio::piped())
                            .spawn()
                            .expect(&format!("出错了! Failed to call eza;"));

                        let mut f_list = File::create(format!("{}{}", f_name, S_ARCHILIST))
                            .expect(&format!("出错了! Failed to create file: {}", f_name));
                        let mut buf = vec![];
                        match do_list
                            .stdout
                            .expect("出错了! Failed to open stdout")
                            .read_to_end(&mut buf)
                        {
                            Err(e) => {
                                eprintln!("出错了! Error, couldn't read stdout: {}", e);
                            }
                            Ok(_) => {
                                let _ = f_list.write_all(&buf);
                            }
                        }
                    }

                    // Compress
                    print!("Compressing: {}", f_name);
                    let f_out = &format!("{}{}", f_name, S_ARCHIVE);
                    let do_compress = Command::new("tar")
                        .arg("-cf")
                        .arg(f_out)
                        .arg(f_name)
                        .spawn()
                        .expect(&format!("出错了! Failed to compress {}", f_name));
                    match do_compress.wait_with_output() {
                        Ok(out) => {
                            if out.status.code() != Some(0) {
                                eprintln!("出错了! tar returned: {:?}", out.status.code());
                                return;
                            }
                        }
                        Err(e) => {
                            eprintln!("出错了! Error with tar compression: {}", e);
                            return;
                        }
                    }
                    print!(" -> {}\n", f_out);

                    // Remove original file
                    let _ = f_remove_print(f_name, f_is_dir);
                }
            }
        }
        None => eprintln!("出错了! Error reading {:?}", entry.file_name()),
    }
}

/// Delete unneeded files, and print any error
fn f_remove_print(f_name: &str, f_is_dir: bool) -> Result<(), std::io::Error> {
    if f_is_dir {
        match remove_dir_all(f_name) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!(
                    "Error, couldn't remove original directory, {}: {}",
                    f_name, e
                );
                Err(e)
            }
        }
    } else {
        match remove_file(f_name) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Error, couldn't remove original file, {}: {}", f_name, e);
                Err(e)
            }
        }
    }
}
