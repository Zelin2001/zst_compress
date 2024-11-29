use std::process::ExitCode;

fn main() -> ExitCode {
    match zst_compress::batch::batch_archive(false) {
        Ok(()) => ExitCode::SUCCESS,
        Err(ret) => ExitCode::from(ret),
    }
}
