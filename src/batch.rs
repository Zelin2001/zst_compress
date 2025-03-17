use crate::runner::Args;
use std::env::{current_dir, current_exe, set_var, var};
use std::fs::{read_dir, remove_dir_all, remove_file, File};
use std::io::{prelude::*, stdout};
use std::path::Path;
use std::process::{Command, Stdio};

// Set the skipped / selected patterns
static S_ARCHIVE: &str = ".tar.zst";
static S_ARCHILIST: &str = "_archived-filelist.txt";
static S_FLAG_MESSAGE: &str = "_archived-message.txt";
static S_TOOL: &str = "zst_";
static S_BIN: &str = ".bin";
static RET_TAR_ERROR: u8 = 1;
static RET_ITEM_ERROR: u8 = 2;
static RET_DIR_ERROR: u8 = 3;

/// Compress or decompress all items in a folder
pub fn batch_archive(args: Args, compress: bool) -> Result<(), u8> {
    let mut ret = 0;
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

    let level_tree = match args.level {
        Some(level) => level as u8,
        None => 4,
    };

    match args.input {
        None => {
            // Walk through videos
            match read_dir(current_dir().unwrap()) {
                Ok(entries) => {
                    for entry in entries {
                        match entry {
                            Ok(entry_file) => {
                                if match entry_file.file_name().to_str() {
                                    Some(f_name) => entry_archive(
                                        f_name,
                                        compress,
                                        args.preserve,
                                        args.flag,
                                        level_tree,
                                        args.target.clone(),
                                    ),
                                    None => {
                                        eprintln!(
                                            "出错了! Error reading: {:?}",
                                            entry_file.file_name()
                                        );
                                        return Err(RET_ITEM_ERROR);
                                    }
                                } != Ok(())
                                {
                                    ret = RET_ITEM_ERROR
                                }
                            }
                            Err(e) => {
                                eprintln!("出错了! Error: {}", e);
                                return Err(RET_DIR_ERROR);
                            }
                        }
                    }
                }
                Err(e) => eprintln!("出错了! Error: {}", e),
            }
        }
        Some(s) => {
            if entry_archive(
                &s,
                compress,
                args.preserve,
                args.flag,
                level_tree,
                args.target.clone(),
            ) != Ok(())
            {
                ret = RET_ITEM_ERROR
            }
        }
    }

    match ret {
        0 => Ok(()),
        _ => Err(ret),
    }
}

/// Compress or decompress 1 item
pub fn entry_archive(
    f_name: &str,
    compress: bool,
    preserve: bool,
    flag: bool,
    level_tree: u8,
    target_dir: Option<String>,
) -> Result<(), u8> {
    let mut ret = 0;

    // Check if is directory
    let f_is_dir = Path::new(f_name).is_dir();
    // Add slash to target_dir
    let target_dir = match target_dir {
        Some(target)
            if (target.rfind("\\") != Some(target.len() - 1) && cfg!(target_os = "windows")) =>
        {
            Some(target + "\\")
        }
        Some(target)
            if (target.rfind("/") != Some(target.len() - 1) && !cfg!(target_os = "windows")) =>
        {
            Some(target + "/")
        }
        _ => target_dir,
    };

    // Skip filelists and tools
    if f_name.find(S_TOOL) == Some(0)
        || f_name.find(S_BIN) == Some(0)
        || (f_name.len() >= S_ARCHILIST.len()
            && f_name.rfind(S_ARCHILIST) == Some(f_name.len() - S_ARCHILIST.len()))
        || (f_name.len() >= S_FLAG_MESSAGE.len()
            && f_name.rfind(S_FLAG_MESSAGE) == Some(f_name.len() - S_FLAG_MESSAGE.len()))
    { // Do nothing
    }
    // Selected archive files
    else if f_name.len() >= S_ARCHIVE.len()
        && f_name.rfind(S_ARCHIVE) == Some(f_name.len() - S_ARCHIVE.len())
    {
        // Decompress and clean
        if !compress {
            print!("Extract: {}", f_name);
            let _ = stdout().flush();
            let f_ori: &str = match target_dir.clone() {
                Some(target) => &(target + &f_name[0..f_name.rfind(S_ARCHIVE).unwrap()]),
                None => &f_name[0..f_name.rfind(S_ARCHIVE).unwrap()],
            };
            let mut command = Command::new("tar");
            let command = match target_dir.clone() {
                Some(target) => command.arg("-xf").arg(f_name).arg("-C").arg(target),
                None => command.arg("-xf").arg(f_name),
            };
            let do_extract = command
                .spawn()
                .expect(&format!("出错了! Failed to extract {}", f_name));

            match do_extract.wait_with_output() {
                Ok(out) => {
                    if out.status.code() != Some(0) {
                        eprintln!("出错了! tar returned: {:?}", out.status.code());
                        return Err(RET_TAR_ERROR);
                    }
                }
                Err(e) => {
                    eprintln!("出错了! Error with tar compression: {}", e);
                    return Err(RET_TAR_ERROR);
                }
            }
            print!(" -> {}\n", f_ori);

            // Remove original file
            if !preserve {
                let _ = f_remove_print(f_name, false);
                let f_list: &str = &format!("{}{}", f_ori, S_ARCHILIST);
                if Path::exists(Path::new(f_list)) {
                    let _ = f_remove_print(f_list, false);
                }
                let f_id: &str = &format!("{}{}", f_ori, S_FLAG_MESSAGE);
                if Path::exists(Path::new(f_id)) {
                    let _ = f_remove_print(f_id, false);
                }
            }
        } else {
            println!("Skip: {}", f_name);
        }
    }
    // Compress, mark the filelist and clean
    else {
        if compress {
            // Make filelist
            if f_is_dir {
                let do_list = Command::new("eza")
                    .arg("-lT")
                    .arg(format!("-L{}", level_tree))
                    .arg(f_name)
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect(&format!("出错了! Failed to call eza;"));

                let f_list_name = match target_dir.clone() {
                    Some(target) => &format!("{}{}{}", target, f_name, S_ARCHILIST),
                    None => &format!("{}{}", f_name, S_ARCHILIST),
                };
                let mut f_list = File::create(f_list_name)
                    .expect(&format!("出错了! Failed to create file: {}", f_name));
                let mut buf = vec![];
                match do_list
                    .stdout
                    .expect("出错了! Failed to open stdout")
                    .read_to_end(&mut buf)
                {
                    Err(e) => {
                        eprintln!("出错了! Error, couldn't read stdout: {}", e);
                        ret = RET_ITEM_ERROR;
                    }
                    Ok(_) => {
                        let _ = f_list.write_all(&buf);
                    }
                }
            }

            // Compress
            print!("Compress: {}", f_name);
            let _ = stdout().flush();
            let f_out: &str = match target_dir.clone() {
                Some(target) => &format!("{}{}{}", target, f_name, S_ARCHIVE),
                None => &format!("{}{}", f_name, S_ARCHIVE),
            };
            let do_compress = Command::new("tar")
                .arg("--zstd")
                .arg("-cf")
                .arg(f_out)
                .arg(f_name)
                .spawn()
                .expect(&format!("出错了! Failed to compress {}", f_name));
            match do_compress.wait_with_output() {
                Ok(out) => {
                    if out.status.code() != Some(0) {
                        eprintln!("出错了! tar returned: {:?}", out.status.code());
                        return Err(RET_TAR_ERROR);
                    }
                }
                Err(e) => {
                    eprintln!("出错了! Error with tar compression: {}", e);
                    return Err(RET_TAR_ERROR);
                }
            }
            print!(" -> {}\n", f_out);

            // Write the indicator text message
            if flag {
                let f_name_id = f_name.to_owned() + S_FLAG_MESSAGE;
                let mut f_id = File::create(&f_name_id)
                    .expect(&format!("出错了! Failed to create file: {}", &f_name_id));
                let message = format!(
                    "- 这是一则数据整理的消息

- 原数据已经压缩，可能移动到新位置: 
  {}
",
                    f_out
                );
                f_id.write_all(message.as_bytes()).expect(&format!(
                    "出错了! Failed to write into file: {}",
                    &f_name_id
                ));
            }

            // Remove original file
            assert!(Path::new(f_name).exists());
            assert!(Path::new(f_out).is_file());
            if !preserve {
                let _ = f_remove_print(f_name, f_is_dir);
            }
        } else {
            println!("Skip: {}", f_name);
        }
    }

    match ret {
        0 => Ok(()),
        ret => Err(ret),
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
