use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pressflow", about = "linux key-chord launcher and HUD daemon")]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    // Toggle HUD
    Toggle,
    // Open HUD
    Show,
    // Close HUD
    Hide,
    // Validate configuration file
    Check {
        path: Option<PathBuf>,
    },

    // Open settings window
    #[command(visible_alias = "preferences")]
    Settings,
}
