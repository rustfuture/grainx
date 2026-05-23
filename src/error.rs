use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GrainxError {
    #[error("grainx monitor requires an interactive terminal. Use: grainx agent")]
    NoTty,

    #[error("invalid bind address: {0}")]
    InvalidBind(String),

    #[error("failed to load config from {path}: {source}")]
    ConfigLoad {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("remote metrics: {0}")]
    RemoteMetrics(String),

    #[error("export failed: {0}")]
    Export(String),

    #[error(transparent)]
    Io(#[from] io::Error),
}

impl GrainxError {
    pub fn exit_code(&self) -> i32 {
        match self {
            GrainxError::NoTty => 2,
            GrainxError::InvalidBind(_) => 2,
            GrainxError::ConfigLoad { .. } => 3,
            GrainxError::RemoteMetrics(_) => 4,
            GrainxError::Export(_) => 5,
            GrainxError::Io(err) => match err.kind() {
                io::ErrorKind::NotFound => 3,
                io::ErrorKind::PermissionDenied => 6,
                io::ErrorKind::InvalidInput => 2,
                io::ErrorKind::NotConnected => 4,
                _ => 1,
            },
        }
    }
}

impl From<GrainxError> for io::Error {
    fn from(err: GrainxError) -> Self {
        io::Error::other(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, GrainxError>;
