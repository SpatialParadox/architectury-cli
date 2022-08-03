use std::io::{stdout, Write};

use clap::Args;
use color_eyre::{eyre::eyre, Result};

use crate::web::WebClient;

#[derive(Args, Debug)]
#[clap(
    alias = "ls",
    about = "List all templates including supported Minecraft versions"
)]
pub struct ListSubcommand {
    #[clap(short, long, value_parser, help = "Name of template to search for")]
    name: Option<String>,
}

impl ListSubcommand {
    pub fn run(self) -> Result<()> {
        let client = WebClient::new();
        let release = client.get_latest_release()?;

        let max_len = release
            .get_templates()
            .map(|s| s.len())
            .max()
            .ok_or_else(|| eyre!("no templates found"))?
            + 2;

        let templates: Vec<String> = if self.name.is_some() {
            let query = self.name.unwrap();
            let matches = release
                .get_templates()
                .cloned()
                .filter(|t| t.contains(&query))
                .collect::<Vec<String>>();

            if matches.is_empty() {
                Err(eyre!("no matching templates found"))
            } else {
                Ok(matches)
            }
        } else {
            Ok(release.get_templates().cloned().collect())
        }?;

        for template in templates {
            let mut versions = release
                .get_template_versions(&template)?
                .cloned()
                .collect::<Vec<String>>();

            versions.sort();

            print!("{}: ", template);
            stdout().flush()?;

            println!(
                "{}{}",
                " ".repeat(max_len - (template.len() + 2)),
                versions.join(", ")
            );
        }

        Ok(())
    }
}
