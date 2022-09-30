use std::{
    collections::{hash_map::Keys, HashMap},
    fs::File,
    io::Write,
};

use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use reqwest::{blocking::Client, header::USER_AGENT};
use serde::Deserialize;
use tempfile::tempfile;

#[derive(Deserialize)]
struct ReleaseData {
    pub assets: Vec<AssetData>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AssetData {
    pub url: String,
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct AssetDownloadData {
    browser_download_url: String,
}

#[derive(Clone)]
pub struct Release {
    assets: HashMap<String, HashMap<String, AssetData>>,
}

pub struct WebClient {
    client: Client,
}

impl Release {
    fn new(release: ReleaseData) -> Result<Self> {
        let mut assets = HashMap::<String, HashMap<String, AssetData>>::new();
        for asset in &release.assets {
            let dash_pos = asset.name.chars().position(|c| c == '-').unwrap_or(0);

            let version = asset.name[..dash_pos].to_owned();

            let filename = &asset.name[dash_pos + 1..asset.name.len() - 4];
            let mixin = filename.ends_with("mixin");

            // Old versions of the template have a different naming scheme, so set them accordingly
            let template = if filename == "mixin" {
                "forge-fabric-mixin".to_owned()
            } else if dash_pos > 0 {
                if filename.starts_with("architectury") {
                    if mixin {
                        "forge-fabric-mixin".to_owned()
                    } else {
                        "forge-fabric".to_owned()
                    }
                } else {
                    filename.to_owned()
                }
            } else {
                "forge-fabric".to_owned()
            };

            let map = assets.entry(template).or_insert_with(HashMap::new);
            map.insert(version, asset.clone());
        }

        Ok(Self { assets })
    }

    pub fn is_supported_version(&self, template: &str, version: &str) -> Result<bool> {
        Ok(self
            .assets
            .get(template)
            .ok_or_else(|| eyre!("no template with name '{}' exists", template))?
            .contains_key(version))
    }

    pub fn get_asset(&self, template: &str, version: &str) -> Result<&AssetData> {
        self.assets
            .get(template)
            .ok_or_else(|| eyre!("no template with name '{}' exists", template))?
            .get(version)
            .ok_or_else(|| eyre!("the {} template does not support {}", template, version))
    }

    pub fn get_templates(&self) -> Keys<String, HashMap<String, AssetData>> {
        self.assets.keys()
    }

    pub fn get_template_versions(&self, template: &str) -> Result<Keys<String, AssetData>> {
        Ok(self
            .assets
            .get(template)
            .ok_or_else(|| eyre!("template does not exist"))?
            .keys())
    }
}

impl WebClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn download_asset(&self, asset: &AssetData) -> Result<File> {
        let asset_data_response = self
            .client
            .get(&asset.url)
            .header(USER_AGENT, "Architectury-CLI")
            .send()
            .wrap_err("failed to fetch asset data")?;

        let asset_data: AssetDownloadData = serde_json::from_str(&asset_data_response.text()?)?;

        let download_response = self
            .client
            .get(&asset_data.browser_download_url)
            .header(USER_AGENT, "Architectury-CLI")
            .send()
            .wrap_err("failed to download asset")?;

        let mut dest = tempfile()?;
        let content = download_response.bytes()?;
        dest.write_all(&content)?;
        Ok(dest)
    }

    pub fn get_latest_release(&self) -> Result<Release> {
        Ok(self
            .get_releases()
            .wrap_err("failed to get releases")?
            .get(0)
            .ok_or_else(|| eyre!("no releases found"))?
            .to_owned())
    }

    pub fn get_releases(&self) -> Result<Vec<Release>> {
        let response_text = self
            .client
            .get("https://api.github.com/repos/architectury/architectury-templates/releases")
            .header(USER_AGENT, "Architectury-CLI")
            .send()?
            .text()?;

        let response_data: Vec<ReleaseData> = serde_json::from_str(&response_text)?;
        if response_data.is_empty() {
            return Err(eyre!("no releases found"));
        }

        let mut releases: Vec<Release> = vec![];
        for release_data in response_data {
            releases.push(Release::new(release_data)?)
        }

        Ok(releases)
    }
}
