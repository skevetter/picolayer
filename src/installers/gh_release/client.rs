use anyhow::Result;
use log::info;
use octocrab::models::repos::Release;

use crate::cli::RetryConfig;
use crate::utils::retry::retry_async;

pub async fn fetch_release(
    owner: &str,
    repo: &str,
    version: &str,
    include_prerelease: bool,
    retry_config: &RetryConfig,
) -> Result<Release> {
    let octocrab = if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        octocrab::Octocrab::builder()
            .personal_token(token)
            .build()?
    } else {
        (*octocrab::instance()).clone()
    };

    if version == "latest" {
        if include_prerelease {
            retry_async(
                retry_config,
                "GitHub API - fetch latest release",
                || async { Ok(octocrab.repos(owner, repo).releases().get_latest().await?) },
            )
            .await
        } else {
            let releases =
                retry_async(retry_config, "GitHub API - fetch releases list", || async {
                    Ok(octocrab.repos(owner, repo).releases().list().send().await?)
                })
                .await?;

            let stable_release = releases
                .items
                .into_iter()
                .find(|r| !r.prerelease)
                .ok_or_else(|| anyhow::anyhow!("No stable releases found"))?;

            info!(
                "Skipping prereleases, using stable release: {}",
                stable_release.tag_name
            );
            Ok(stable_release)
        }
    } else {
        retry_async(
            retry_config,
            "GitHub API - fetch release by tag",
            || async {
                Ok(octocrab
                    .repos(owner, repo)
                    .releases()
                    .get_by_tag(version)
                    .await?)
            },
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use octocrab::models::repos::Release;

    fn create_mock_release(tag_name: &str, prerelease: bool) -> Release {
        Release {
            id: 1.into(),
            node_id: "node123".to_string(),
            tag_name: tag_name.to_string(),
            target_commitish: "main".to_string(),
            name: Some(tag_name.to_string()),
            body: None,
            draft: false,
            prerelease,
            created_at: Some(Utc::now()),
            published_at: Some(Utc::now()),
            author: None,
            assets: vec![],
            upload_url: "https://example.com/upload".to_string(),
            html_url: "https://example.com".parse().unwrap(),
            assets_url: "https://example.com/assets".parse().unwrap(),
            tarball_url: Some("https://example.com/tarball".parse().unwrap()),
            zipball_url: Some("https://example.com/zipball".parse().unwrap()),
            url: "https://example.com/release".parse().unwrap(),
        }
    }

    #[test]
    fn test_mock_release_creation() {
        let release = create_mock_release("v1.0.0", false);
        assert_eq!(release.tag_name, "v1.0.0");
        assert!(!release.prerelease);
    }

    #[test]
    fn test_mock_prerelease_creation() {
        let release = create_mock_release("v1.0.0-beta", true);
        assert_eq!(release.tag_name, "v1.0.0-beta");
        assert!(release.prerelease);
    }
}
