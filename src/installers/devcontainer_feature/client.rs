use anyhow::{Context, Result};
use log::info;
use oci_client::{Client, Reference, client::ClientConfig};
use std::io::Cursor;
use std::path::Path;

use crate::cli::RetryConfig;
use crate::utils::retry::retry_async;

/// Download and extract OCI layer
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

    let config = ClientConfig {
        accept_invalid_certificates: false,
        accept_invalid_hostnames: false,
        ..Default::default()
    };

    let client = Client::new(config);

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

    let accepted_media_types = vec![
        "application/vnd.devcontainers.layer.v1+tar",
        "application/vnd.oci.image.layer.v1.tar",
        "application/vnd.oci.image.layer.v1.tar+gzip",
        "application/vnd.docker.image.rootfs.diff.tar",
        "application/vnd.docker.image.rootfs.diff.tar.gzip",
    ];

    let image_data = retry_async(retry_config, "OCI image pull", || async {
        client
            .pull(&reference, &auth, accepted_media_types.clone())
            .await
            .with_context(|| format!("Failed to pull OCI image: {}", reference))
    })
    .await?;

    let layer = image_data
        .layers
        .first()
        .ok_or_else(|| anyhow::anyhow!("Feature OCI image has no layers"))?;

    let is_gzipped = layer.data.len() >= 2 && layer.data[0] == 0x1f && layer.data[1] == 0x8b;
    info!(
        "Extracting layer with {} bytes (gzipped: {})",
        layer.data.len(),
        is_gzipped
    );

    if is_gzipped {
        let decoder = flate2::read::GzDecoder::new(&layer.data[..]);
        let mut archive = tar::Archive::new(decoder);
        archive
            .unpack(output_dir)
            .context("Failed to extract gzipped layer archive")?;
    } else {
        let cursor = Cursor::new(&layer.data);
        let mut archive = tar::Archive::new(cursor);
        archive
            .unpack(output_dir)
            .context("Failed to extract plain tar layer archive")?;
    }

    Ok(())
}
