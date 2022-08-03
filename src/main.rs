use clap::{Parser, Subcommand};
use color_eyre::Result;
use commands::{list::ListSubcommand, new::NewSubcommand};

mod commands;
mod web;

#[derive(Parser, Debug)]
#[clap(version = "1.0")]
struct Cli {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    New(NewSubcommand),
    List(ListSubcommand),
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    match cli.command {
        Subcommands::New(subcommand) => subcommand.run(),
        Subcommands::List(subcommand) => subcommand.run(),
    }?;

    Ok(())
}
