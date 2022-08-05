use std::{
    fs::{self, File},
    io,
    path::PathBuf,
};

use clap::Args;
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use zip::ZipArchive;

use crate::web::WebClient;

#[derive(Debug)]
enum TemplateType {
    Forge,
    ForgeFabric(bool),
    ForgeQuilt(bool),
    ForgeFabricLike(bool),
    ForgeFabricQuilt(bool),
}

impl TemplateType {
    pub fn from_str(s: &str, mixin: bool) -> Result<Self> {
        match s {
            "forge" => Ok(TemplateType::Forge),
            "forge-fabric" => Ok(TemplateType::ForgeFabric(mixin)),
            "forge-quilt" => Ok(TemplateType::ForgeQuilt(mixin)),
            "forge-fabric-like" => Ok(TemplateType::ForgeFabricLike(mixin)),
            "forge-fabric-quilt" => Ok(TemplateType::ForgeFabricQuilt(mixin)),
            _ => Err(eyre!("invalid template type: {}", s)),
        }
    }
}

#[derive(Args, Debug)]
#[clap(about = "Create a new project from a template")]
pub struct NewSubcommand {
    #[clap(short, long, help = "Minecraft version")]
    version: String,

    #[clap(short, long, action, help = "Use mixin version of template")]
    mixin: bool,

    #[clap(short, long, help = "Template name")]
    template: String,

    #[clap(help = "Project directory to create")]
    directory: PathBuf,
}

impl NewSubcommand {
    pub fn run(self) -> Result<()> {
        if self.directory.file_name().is_none() {
            return Err(eyre!("invalid path given: {:?}", self.directory));
        }

        if self.directory.exists() {
            return Err(eyre!("{:?} already exists", self.directory));
        }

        let client = WebClient::new();
        let release = client
            .get_latest_release()
            .wrap_err("failed to get latest release")?;

        let template_type = TemplateType::from_str(self.template.as_str(), self.mixin)?;
        let template_name = match template_type {
            TemplateType::Forge => self.template.clone(),
            _ => {
                let mut name = self.template.clone();
                if self.mixin {
                    name += "-mixin"
                }
                name
            }
        };

        if !release.is_supported_version(&template_name, &self.version)? {
            return Err(eyre!(
                "invalid Minecraft version provided: {}, see versions subcommand",
                self.version
            ));
        }

        let asset = release.get_asset(&template_name, &self.version)?;

        let asset_file = client
            .download_asset(asset)
            .wrap_err("failed to download asset")?;

        println!(
            "Successfully downloaded {} {} template, extracting...",
            self.version, template_name
        );

        // Create project directory and extract all archive files into it
        let mut archive = ZipArchive::new(asset_file)?;
        for i in 0..archive.len() {
            let mut archive_file = archive.by_index(i)?;
            let file_path = match archive_file.enclosed_name() {
                Some(path) => self.directory.join(path),
                None => continue,
            };

            if archive_file.name().ends_with('/') {
                fs::create_dir_all(&file_path)?;
            } else if let Some(p) = file_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }

                let mut new_file = File::create(&file_path)?;
                io::copy(&mut archive_file, &mut new_file)?;
            }

            // set permissions for unix systems
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = archive_file.unix_mode() {
                    fs::set_permissions(&file_path, fs::Permissions::from_mode(mode))?;
                }
            }
        }

        println!("Project created at {:?}", self.directory);

        Ok(())
    }
}
