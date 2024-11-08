use std::fmt;
use std::{io, path::PathBuf};

pub type Result<T> = std::result::Result<T, BuildError>;

#[derive(Debug)]
pub enum BuildError {
    EnvVar(std::env::VarError),
    Io(io::Error),
    LibraryNotFound(PathBuf),
    RssPathNotFound,
    StubGenerationFailed(String),
    BindgenError(String),
    PythonError(String),
    CompilationError(String),
    HeadersNotFound(PathBuf),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::EnvVar(e) => write!(f, "Environment variable error: {}", e),
            BuildError::Io(e) => write!(f, "IO error: {}", e),
            BuildError::LibraryNotFound(path) => {
                write!(f, "Library not found at: {}", path.display())
            }
            BuildError::RssPathNotFound => write!(f, "RSS directory not found"),
            BuildError::StubGenerationFailed(msg) => write!(f, "Stub generation failed: {}", msg),
            BuildError::BindgenError(msg) => write!(f, "Bindgen error: {}", msg),
            BuildError::PythonError(msg) => write!(f, "Python script error: {}", msg),
            BuildError::CompilationError(msg) => write!(f, "Compilation error: {}", msg),
            BuildError::HeadersNotFound(path) => {
                write!(f, "Headers not found at: {}", path.display())
            }
        }
    }
}

impl std::error::Error for BuildError {}

impl From<io::Error> for BuildError {
    fn from(err: io::Error) -> Self {
        BuildError::Io(err)
    }
}
