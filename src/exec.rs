use std::cmp::max;
use std::fs::{File, remove_dir_all, remove_file};
use std::io::{copy, prelude::*, stdout};
use std::path::Path;
use std::thread;

// Set the skipped / selected patterns
static S_ARCHIVE: &str = ".tar.zst";
static S_ARCHILIST: &str = "_archived-filelist.txt";
static S_FLAG_MESSAGE: &str = "_archived-message.txt";
static S_TOOL: &str = "zst_";
static RET_TAR_ERROR: u8 = 1;
pub static RET_ITEM_ERROR: u8 = 2;
pub static RET_DIR_ERROR: u8 = 3;

/// Compress or decompress 1 item
#[allow(clippy::too_many_arguments)]
pub fn entry_archive(
    f_path: &Path,
    compress: bool,
    preserve: bool,
    flag: bool,
    target_dir: &Path,
    level_tree: u8,
    level_zstd: i32,
    current: usize,
    total: usize,
    dry_run: bool,
) -> Result<(), u8> {
    let mut ret = 0;

    // Get clean name and determined target_dir
    let f_name = f_path
        .file_name()
        .ok_or(RET_ITEM_ERROR)?
        .to_str()
        .ok_or(RET_ITEM_ERROR)?;

    // Print progress counting
    print!("({current}/{total}) ");

    // Skip filelists and tools
    if f_name.find(S_TOOL) == Some(0)
        || (f_name.len() >= S_ARCHILIST.len()
            && f_name.rfind(S_ARCHILIST) == Some(f_name.len() - S_ARCHILIST.len()))
        || (f_name.len() >= S_FLAG_MESSAGE.len()
            && f_name.rfind(S_FLAG_MESSAGE) == Some(f_name.len() - S_FLAG_MESSAGE.len()))
    {
        println!("Skip: {:?}", f_path);
    }
    // Selected archive files
    else if f_name.len() >= S_ARCHIVE.len()
        && f_name.rfind(S_ARCHIVE) == Some(f_name.len() - S_ARCHIVE.len())
    {
        // Decompress and clean
        if !compress {
            print!("Extract: {:?}", f_path);
            let _ = stdout().flush();
            let f_ori_name = &f_name[0..f_name.rfind(S_ARCHIVE).unwrap()];
            let f_ori_buf = target_dir.join(f_ori_name);
            let f_ori = f_ori_buf.as_path();
            if !dry_run && do_archive(f_path, target_dir, false, level_zstd).is_err() {
                eprintln!("出错了! Failed to extract {:?}", f_path);
                return Err(RET_TAR_ERROR);
            }
            println!(" -> {:?}", f_ori);

            // Remove original file
            if !preserve && !dry_run {
                let _ = f_remove_print(f_path, false);
                let f_list_buf = f_ori.with_file_name(format!("{f_ori_name}{S_ARCHILIST}"));
                let f_list = f_list_buf.as_path();
                if Path::exists(f_list) {
                    let _ = f_remove_print(f_list, false);
                }
                let f_id_buf = f_ori.with_file_name(format!("{f_ori_name}{S_FLAG_MESSAGE}"));
                let f_id = f_id_buf.as_path();
                if Path::exists(f_id) {
                    let _ = f_remove_print(f_id, false);
                }
            }
        } else {
            println!("Skip: {:?}", f_path);
        }
    }
    // Compress, mark the filelist and clean
    else if compress {
        // Make filelist
        if f_path.is_dir() {
            let f_list_path_buf = target_dir.join(format!("{f_name}{S_ARCHILIST}"));
            let f_list_path = f_list_path_buf.as_path();

            if let Err(e) = dir_listing::generate_listing(f_path, f_list_path, level_tree, dry_run)
            {
                eprintln!(
                    "出错了! Error generating directory listing for {}: {e}",
                    f_path.display()
                );
                ret = RET_ITEM_ERROR;
            }
        }

        // Compress
        print!("Compress: {:?}", f_path);
        let _ = stdout().flush();
        let f_out = target_dir.join(format!("{f_name}{S_ARCHIVE}"));
        if !dry_run && do_archive(f_path, target_dir, true, level_zstd).is_err() {
            eprintln!("出错了! Failed to compress {:?}", f_path);
            return Err(RET_TAR_ERROR);
        }
        println!(" -> {:?}", f_out);

        // Write the indicator text message
        if flag && !dry_run {
            let f_name_id_buf = f_path.with_file_name(format!("{f_name}{S_FLAG_MESSAGE}"));
            let f_name_id = f_name_id_buf.as_path();
            let mut f_id = File::create(f_name_id)
                .unwrap_or_else(|_| panic!("出错了! Failed to create file: {:?}", f_name_id));
            let message = format!(
                "- 这是一则数据整理的消息

    - 原数据已经压缩，可能移动到新位置: 
      {:?}
    ",
                f_out
            );
            f_id.write_all(message.as_bytes())
                .unwrap_or_else(|_| panic!("出错了! Failed to write into file: {:?}", f_name_id));
        }

        // Remove original file
        if !dry_run {
            assert!(f_path.exists());
            assert!(f_out.is_file());
            if !preserve {
                let _ = f_remove_print(f_path, f_path.is_dir());
            }
        }
    } else {
        println!("Skip: {:?}", f_path);
    }

    match ret {
        0 => Ok(()),
        ret => Err(ret),
    }
}

/// Implement compression with archive library tar and zstd
fn do_archive(f_path: &Path, target: &Path, compress: bool, level_zstd: i32) -> Result<(), u8> {
    if compress {
        // Compression path: tar -> zstd
        let output_path = target.join(format!(
            "{}.tar.zst",
            f_path.file_name().unwrap().to_str().unwrap()
        ));
        let output_file = File::create(&output_path).map_err(|_| RET_TAR_ERROR)?;

        let (mut reader, writer) = pipe::pipe();

        // 启动压缩线程
        let compressor = thread::spawn(move || {
            let mut encoder = zstd::stream::Encoder::new(output_file, level_zstd).unwrap();
            let cpus = thread::available_parallelism().unwrap().get();
            encoder.multithread(max(cpus as u32 / 2, 10)).unwrap();
            copy(&mut reader, &mut encoder).unwrap();
            encoder.finish().unwrap();
        });

        // 主线程生成 tar
        {
            let mut builder = tar::Builder::new(writer);
            if f_path.is_dir() {
                builder
                    .append_dir_all(f_path.file_name().unwrap(), f_path)
                    .map_err(|_| RET_TAR_ERROR)?;
            } else {
                builder
                    .append_path_with_name(f_path, f_path.file_name().unwrap())
                    .map_err(|_| RET_TAR_ERROR)?;
            }
            builder.finish().map_err(|_| RET_TAR_ERROR)?;
        }

        compressor.join().map_err(|_| RET_TAR_ERROR)?;
    } else {
        // Decompression path: zstd -> tar file -> unpack
        let file_stem = f_path.file_stem().unwrap().to_str().unwrap();
        let tar_path = target.join(format!("{file_stem}.tar"));

        // First decompress to .tar file
        {
            let input_file = File::open(f_path).map_err(|_| RET_TAR_ERROR)?;
            let output_file = File::create(&tar_path).map_err(|_| RET_TAR_ERROR)?;
            zstd::stream::copy_decode(input_file, output_file).map_err(|_| RET_TAR_ERROR)?;
        }

        // Then unpack the tar file
        let tar_file = File::open(&tar_path).map_err(|_| RET_TAR_ERROR)?;
        let mut archive = tar::Archive::new(tar_file);
        archive.unpack(target).map_err(|_| RET_TAR_ERROR)?;

        // Clean up the intermediate tar file
        std::fs::remove_file(&tar_path).map_err(|_| RET_TAR_ERROR)?;
    }

    Ok(())
}

/// Listing files in a directory to be compressed
mod dir_listing {
    use std::fs::{self, DirEntry};
    use std::io::{self, Write};
    use std::path::Path;
    use std::time::SystemTime;

    pub fn generate_listing(
        dir_path: &Path,
        output_path: &Path,
        max_depth: u8,
        dry_run: bool,
    ) -> Result<(), io::Error> {
        let mut output: Box<dyn Write> = match dry_run {
            false => Box::new(fs::File::create(output_path)?),
            true => Box::new(io::sink()),
        };
        list_directory(dir_path, &mut output, max_depth, 0)
    }

    fn list_directory(
        path: &Path,
        output: &mut dyn Write,
        max_depth: u8,
        current_depth: u8,
    ) -> io::Result<()> {
        if current_depth > max_depth {
            return Ok(());
        }

        let entries = fs::read_dir(path)?;
        let mut entries: Vec<DirEntry> = entries.filter_map(Result::ok).collect();
        entries.sort_by_key(|a| a.file_name());

        for entry in entries {
            let metadata = entry.metadata()?;
            let file_type = metadata.file_type();
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            // Skip hidden files/directories
            if file_name.starts_with('.') {
                continue;
            }

            // Tree prefix with proper characters
            let tree_prefix = if current_depth == 0 {
                String::new()
            } else {
                let mut prefix = String::new();
                for i in 0..current_depth {
                    if i == current_depth - 1 {
                        prefix.push_str("└──");
                    } else {
                        prefix.push_str("│  ");
                    }
                }
                prefix
            };

            let size = if file_type.is_dir() {
                let dir_size = dir_size(&entry.path())?;
                format!("{:>10}", human_size(dir_size))
            } else {
                let file_size = metadata.len();
                format!("{:>10}", human_size(file_size))
            };

            let modified = metadata.modified()?;
            let modified = system_time_to_date_time(modified);

            // Format with date and size on left, tree on right
            writeln!(
                output,
                "{:<19} {:>10} {}{} {}",
                modified,
                size,
                tree_prefix,
                if file_type.is_dir() { "┬" } else { "─" },
                file_name
            )?;

            if file_type.is_dir() {
                list_directory(&entry.path(), output, max_depth, current_depth + 1)?;
            }
        }

        Ok(())
    }

    fn dir_size(path: &Path) -> io::Result<u64> {
        fn walk_dir(path: &Path, total: &mut u64) -> io::Result<()> {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let metadata = entry.metadata()?;

                if metadata.is_dir() {
                    walk_dir(&entry.path(), total)?;
                } else {
                    *total += metadata.len();
                }
            }
            Ok(())
        }

        let mut total = 0;
        walk_dir(path, &mut total)?;
        Ok(total)
    }

    fn human_size(size: u64) -> String {
        const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
        let mut size = size as f64;
        let mut unit_idx = 0;

        while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }

        format!("{:.1}{}", size, UNITS[unit_idx])
    }

    fn system_time_to_date_time(time: SystemTime) -> String {
        use chrono::{DateTime, Local};
        let datetime: DateTime<Local> = time.into();
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

/// Delete unneeded files, and print any error
fn f_remove_print(f_path: &Path, f_is_dir: bool) -> Result<(), std::io::Error> {
    if f_is_dir {
        match remove_dir_all(f_path) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!(
                    "Error, couldn't remove original directory, {:?}: {e}",
                    f_path
                );
                Err(e)
            }
        }
    } else {
        match remove_file(f_path) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Error, couldn't remove original file, {:?}: {e}", f_path);
                Err(e)
            }
        }
    }
}
