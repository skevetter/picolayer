use anyhow::{Context, Result};
use log::info;
use oci_client::{Client, Reference};
use std::path::Path;

use crate::cli::RetryConfig;
use crate::utils::retry::retry_async;

/// Download and extract OCI layer using oci_client with retry logic
pub async fn download_and_extract_layer(
    feature_ref: &str,
    output_dir: &Path,
    username: Option<&str>,
    password: Option<&str>,
    token: Option<&str>,
    retry_config: &RetryConfig,
) -> Result<()> {
    let reference: Reference = feature_ref
        .parse()
        .with_context(|| format!("Invalid OCI reference: {}", feature_ref))?;

    info!("Parsed OCI reference: {}", reference);

    let client = Client::new(Default::default());

    info!("Pulling OCI image: {}", reference);

    let auth = match (token, username, password) {
        (Some(token), _, _) => {
            info!("Using bearer token authentication for registry");
            oci_client::secrets::RegistryAuth::Bearer(token.to_string())
        }
        (None, Some(user), Some(pass)) => {
            info!("Using basic authentication for registry");
            oci_client::secrets::RegistryAuth::Basic(user.to_string(), pass.to_string())
        }
        _ => {
            info!("Using anonymous authentication for registry");
            oci_client::secrets::RegistryAuth::Anonymous
        }
    };

    let image_data = retry_async(retry_config, "OCI image pull", || async {
        client
            .pull(&reference, &auth, vec![])
            .await
            .with_context(|| format!("Failed to pull OCI image: {}", reference))
    })
    .await?;

    if image_data.layers.is_empty() {
        anyhow::bail!("Feature OCI image has no layers");
    }

    // Extract the first layer (devcontainer features typically have one layer)
    let layer = &image_data.layers[0];
    info!("Extracting layer with {} bytes", layer.data.len());

    let decoder = flate2::read::GzDecoder::new(&layer.data[..]);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(output_dir)
        .context("Failed to extract layer archive")?;

    Ok(())
}
