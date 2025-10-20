use std::fmt;

#[derive(Debug)]
pub enum PicolayerError {
    RepositoryNotFound,
    ContainerFeatureDownloadFailed,
    NoMatchingAssets,
    PermissionDenied,
    InsufficientDiskSpace,
    NetworkConnectionFailed,
    CatchAll(anyhow::Error),
}

impl fmt::Display for PicolayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PicolayerError::RepositoryNotFound => {
                writeln!(f, "Error: Repository not found or not accessible")?;
                write!(
                    f,
                    "Check the owner/repo names and ensure the repository exists"
                )
            }
            PicolayerError::ContainerFeatureDownloadFailed => {
                writeln!(f, "Error: Failed to download container feature")?;
                write!(f, "Check the feature reference and your network connection")
            }
            PicolayerError::NoMatchingAssets => {
                writeln!(f, "Error: No matching release assets found")?;
                write!(f, "Check your filter criteria or try a different version")
            }
            PicolayerError::PermissionDenied => {
                writeln!(f, "Error: Permission denied")?;
                write!(
                    f,
                    "Check file permissions or run with appropriate privileges"
                )
            }
            PicolayerError::InsufficientDiskSpace => {
                writeln!(f, "Error: Insufficient disk space")?;
                write!(f, "Free up disk space and try again")
            }
            PicolayerError::NetworkConnectionFailed => {
                writeln!(f, "Error: Network connection failed")?;
                write!(f, "Check your internet connection and try again")
            }
            PicolayerError::CatchAll(e) => {
                writeln!(f, "Error: {}", e)?;
                if std::env::var("RUST_BACKTRACE").is_ok()
                    || std::env::var("PICOLAYER_DEBUG").is_ok()
                {
                    writeln!(f, "\nTechnical details:")?;
                    write!(f, "{:?}", e)
                } else {
                    write!(f, "\nFor technical details, set PICOLAYER_DEBUG=1")
                }
            }
        }
    }
}

impl From<anyhow::Error> for PicolayerError {
    fn from(error: anyhow::Error) -> Self {
        let full_error = format!("{:?}", error);

        if (error.to_string().contains("GitHub") || full_error.contains("GitHub"))
            && (full_error.contains("Not Found") || full_error.contains("not found"))
        {
            PicolayerError::RepositoryNotFound
        } else if full_error.contains("Failed to pull OCI image")
            || full_error.contains("Not authorized")
        {
            PicolayerError::ContainerFeatureDownloadFailed
        } else if full_error.contains("No matching")
            || full_error.contains("filter")
            || full_error.contains("No suitable asset found")
        {
            PicolayerError::NoMatchingAssets
        } else if full_error.contains("Permission denied") || full_error.contains("Access denied") {
            PicolayerError::PermissionDenied
        } else if full_error.contains("No space left") {
            PicolayerError::InsufficientDiskSpace
        } else if full_error.contains("Network")
            || full_error.contains("connection")
            || full_error.contains("timeout")
        {
            PicolayerError::NetworkConnectionFailed
        } else {
            PicolayerError::CatchAll(error)
        }
    }
}
