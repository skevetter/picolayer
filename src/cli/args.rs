use clap::{Parser, Subcommand};
use std::collections::HashMap;

#[derive(Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl RetryConfig {
    pub fn from_cli(cli: &Cli) -> Self {
        Self {
            max_retries: cli.max_retries,
            initial_delay_ms: cli.retry_delay_ms,
            backoff_multiplier: cli.retry_backoff_multiplier,
        }
    }
}

#[derive(Parser)]
#[command(name = "picolayer")]
#[command(about = "Ensures minimal container layers")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Maximum number of retry attempts for downloads (default: 0, no retries)
    #[arg(long, global = true, default_value = "0")]
    pub max_retries: u32,

    /// Initial delay in milliseconds for retry backoff (default: 1000)
    #[arg(long, global = true, default_value = "1000")]
    pub retry_delay_ms: u64,

    /// Multiplier for exponential backoff (default: 2.0)
    #[arg(long, global = true, default_value = "2.0")]
    pub retry_backoff_multiplier: f64,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install packages using apt-get
    #[command(name = "apt-get")]
    AptGet {
        /// Comma-separated list of packages to install
        packages: String,

        #[command(flatten)]
        ppa_args: PpaArgs,
    },

    /// Install packages using apt
    Apt {
        /// Comma-separated list of packages to install
        packages: String,

        #[command(flatten)]
        ppa_args: PpaArgs,
    },

    /// Install packages using aptitude
    Aptitude {
        /// Comma-separated list of packages to install
        packages: String,
    },

    /// Install packages using apk
    Apk {
        /// Comma-separated list of packages to install
        packages: String,
    },

    /// Install packages using Homebrew
    Brew {
        /// Comma-separated list of packages to install
        packages: String,
    },

    /// Install npm packages
    Npm {
        /// Comma-separated list of packages to install
        packages: String,
    },

    /// Install Python packages using pipx
    Pipx {
        /// Python version to use (e.g., python3.9, python3.10)
        #[arg(long)]
        python: Option<String>,
        /// Comma-separated list of packages to install
        packages: String,
    },

    /// Install a devcontainer feature
    #[command(name = "devcontainer-feature")]
    DevcontainerFeature {
        /// OCI feature reference (e.g., ghcr.io/devcontainers/features/node:1)
        feature: String,

        /// Feature options (key=value pairs)
        #[arg(long)]
        option: Vec<String>,

        /// Remote user for feature installation
        #[arg(long)]
        remote_user: Option<String>,

        /// Environment variables (key=value pairs)
        #[arg(long)]
        env: Vec<String>,

        /// Script name to execute (default: install.sh)
        #[arg(long, default_value = "install.sh")]
        script: String,

        /// User to install for (overrides automatic detection)
        #[arg(long)]
        user: Option<String>,

        /// Registry username for authentication
        #[arg(long)]
        registry_username: Option<String>,

        /// Registry password for authentication
        #[arg(long)]
        registry_password: Option<String>,

        /// Registry bearer token for authentication
        #[arg(long)]
        registry_token: Option<String>,
    },

    /// Install binary from GitHub release
    #[command(name = "gh-release")]
    GhRelease {
        /// Repository owner
        #[arg(long)]
        owner: String,

        /// Repository name
        #[arg(long)]
        repo: String,

        /// Comma-separated list of binary names
        #[arg(long)]
        binary: Option<String>,

        /// Version to install (default: latest)
        #[arg(long, default_value = "latest")]
        version: String,

        /// Directory to install binaries
        #[arg(long, default_value = "/usr/local/bin")]
        install_dir: String,

        /// Regex pattern for asset filtering
        #[arg(long)]
        filter: Option<String>,

        /// Verify checksums using checksum files
        #[arg(long, default_value = "false", conflicts_with = "checksum_text")]
        verify_checksum: bool,

        /// Checksum text for verification (e.g., "sha256:5d3d3c60ffcf601f964bb4060a4234f9a96a3b09a7cdf67d1e61ae88efcd48f4")
        #[arg(long, conflicts_with = "verify_checksum")]
        checksum_text: Option<String>,

        /// GPG public key for signature verification (can be a URL, file path, or key content)
        #[arg(long)]
        gpg_key: Option<String>,

        /// Include prerelease versions
        #[arg(long, default_value = "false")]
        include_prerelease: bool,
    },

    /// Run a command using pkgx
    Pkgx {
        /// Tool name (e.g., "python", "node", "go")
        #[arg(long)]
        tool: String,

        /// Tool version (e.g., "3.10", "18", "latest")
        #[arg(long, default_value = "latest")]
        version: String,

        /// Arguments to pass to the tool
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,

        /// Working directory for execution
        #[arg(long, default_value = ".")]
        working_dir: String,

        /// Environment variables (key=value pairs)
        #[arg(long)]
        env: Vec<String>,
    },
}

/// Common PPA arguments for apt-based installers
#[derive(clap::Args)]
pub struct PpaArgs {
    /// Comma-separated list of PPAs to use
    #[arg(long)]
    pub ppas: Option<String>,

    /// Force PPAs on non-Ubuntu systems
    #[arg(long, default_value = "false")]
    pub force_ppas_on_non_ubuntu: bool,
}

/// Parse comma-separated string into a vector of trimmed strings
pub fn normalize_package_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse key=value pairs into a HashMap
pub fn parse_key_value_pairs(pairs: &[String]) -> Option<HashMap<String, String>> {
    if pairs.is_empty() {
        return None;
    }

    let map: HashMap<String, String> = pairs
        .iter()
        .filter_map(|pair| {
            pair.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
        })
        .collect();

    if map.is_empty() { None } else { Some(map) }
}
