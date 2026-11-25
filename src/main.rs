use clap::Parser;
use pressflow::{
    cli::{Args, Commands},
    config::validate::validate_config,
};

fn main() {
    let cli = Args::parse();

    match cli.command {
        Some(Commands::Toggle) => {
            print!("toggle");
        }
        Some(Commands::Show) => {
            print!("Show");
        }
        Some(Commands::Hide) => {
            print!("Hide");
        }
        Some(Commands::Check { path }) => {
            validate_config(path);
        }
        Some(Commands::Settings) => {
            print!("Settings")
        }
        _ => {
            print!("None")
        }
    }
}
