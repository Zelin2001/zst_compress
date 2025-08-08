use std::path::{Path, PathBuf};

static RET_DIR_ERROR: u8 = 3;

pub struct DirGuard {
    original: PathBuf,
}

impl DirGuard {
    pub fn new(target: &Path) -> Result<Self, u8> {
        let original = std::env::current_dir().map_err(|_| RET_DIR_ERROR)?;
        std::env::set_current_dir(target).map_err(|_| RET_DIR_ERROR)?;
        Ok(Self { original })
    }
}

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}
