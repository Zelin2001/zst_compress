use std::process::ExitCode;

pub fn cli(compress:bool) -> ExitCode {
    match zst_compress::batch::batch_archive(compress) {
        Ok(()) => ExitCode::SUCCESS,
        Err(ret) => ExitCode::from(ret),
    }
}