use clap::Parser;
use pressflow::cli::{Args, Commands};

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
            print!("Path: {}", path.unwrap().display());
        }
        Some(Commands::Settings) => {
            print!("Settings")
        }
        _ => {
            print!("None")
        }
    }
}
