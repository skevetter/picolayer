mod client;
mod feature;
mod installer;

use anyhow::Result;
use std::collections::HashMap;

pub struct DevcontainerFeatureConfig<'a> {
    pub feature_ref: &'a str,
    pub options: Option<HashMap<String, String>>,
    pub remote_user: Option<&'a str>,
    pub envs: Option<HashMap<String, String>>,
    pub script_name: &'a str,
    pub user: Option<&'a str>,
    pub registry_username: Option<&'a str>,
    pub registry_password: Option<&'a str>,
    pub registry_token: Option<&'a str>,
}

/// Install a devcontainer feature from an OCI reference (async)
pub async fn install_async(
    config: &DevcontainerFeatureConfig<'_>,
    retry_config: &crate::cli::RetryConfig,
) -> Result<()> {
    installer::install_async(config, retry_config).await
}
