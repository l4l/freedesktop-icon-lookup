use std::io::Error as IoError;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("icon theme index is missing: {path:?}")]
    ThemeIndexMissing { path: PathBuf, source: IoError },
    #[error("icon theme index is invalid: {path:?}")]
    InvalidIndex { path: PathBuf, source: tini::Error },
    #[error("error at icon dir traversing")]
    TraverseDir { source: IoError },
    #[error("inheritance cycle detected")]
    CycleDetected,
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
