use std::process::ExitCode;

pub fn cli(compress:bool) -> ExitCode {
    match crate::batch::batch_archive(compress) {
        Ok(()) => ExitCode::SUCCESS,
        Err(ret) => ExitCode::from(ret),
    }
}