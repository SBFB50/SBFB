// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::{Parser, Subcommand};
use std::process;

mod provenance;
mod secret_scanner;
mod template_engine;
mod template_lock;

#[derive(Parser)]
#[command(name = "sbfb-factory", about = "SBFB app scaffolding tool")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new SBFB app from a template
    Create {
        /// Template to use
        #[arg(long, default_value = "static")]
        template: String,

        /// App name
        #[arg(long)]
        name: String,

        /// Output directory (defaults to ./<name>)
        #[arg(long)]
        output: Option<String>,
    },

    /// Validate an existing SBFB project
    Validate {
        /// Path to the project directory
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Create {
            template,
            name,
            output,
        } => {
            let output_dir = output.unwrap_or_else(|| name.clone());
            template_engine::create(&template, &name, &output_dir)
        }
        Command::Validate { path } => template_engine::validate(&path),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
