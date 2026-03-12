mod client;
mod extractor;
mod selector;
mod verifier;

use anyhow::Result;
use log::info;

pub struct GhReleaseConfig<'a> {
    pub owner: &'a str,
    pub repo: &'a str,
    pub binary_names: &'a [String],
    pub version: &'a str,
    pub install_dir: &'a str,
    pub filter: Option<&'a str>,
    pub verify_checksum: bool,
    pub checksum_text: Option<&'a str>,
    pub gpg_key: Option<&'a str>,
    pub include_prerelease: bool,
}

pub async fn install(
    config: &GhReleaseConfig<'_>,
    retry_config: &crate::cli::RetryConfig,
) -> Result<()> {
    info!(
        "Fetching release information for {}/{}",
        config.owner, config.repo
    );

    let release = client::fetch_release(
        config.owner,
        config.repo,
        config.version,
        config.include_prerelease,
        retry_config,
    )
    .await?;
    info!("Installing from release: {}", release.tag_name);

    let selector = selector::create_selector(config.filter);
    let selected = selector.select(&release.assets)?;
    info!("Selected asset: {}", selected.name);

    // When verification is requested and no explicit checksum text is provided,
    // prefer a signed variant if the selected asset has no direct signature.
    let asset = if config.verify_checksum && config.checksum_text.is_none() {
        if let Some(signed_variant) = verifier::find_signed_variant(&release.assets, selected) {
            info!("Using signed variant for verification: {}", signed_variant.name);
            signed_variant
        } else {
            selected
        }
    } else {
        selected
    };

    if let Some(checksum_text) = config.checksum_text {
        verifier::verify_with_checksum_text(asset, checksum_text).await?;
    } else if config.verify_checksum {
        verifier::verify_asset(&release.assets, asset, config.gpg_key).await?;
    }

    extractor::extract_and_install(asset, config.binary_names, config.install_dir).await?;

    info!("Installation complete!");
    Ok(())
}
