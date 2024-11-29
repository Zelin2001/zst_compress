use std::process::ExitCode;

fn main() -> ExitCode {
    match zst_compress::batch::batch_archive(true) {
        Ok(()) => ExitCode::SUCCESS,
        Err(ret) => ExitCode::from(ret),
    }
}
