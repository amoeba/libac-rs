pub mod cli_helper;

use std::error::Error;

use clap::{Parser, Subcommand};

use crate::cli_helper::extract_texture_by_id;

#[derive(Parser)]
#[command(name = "dat")]
#[command(about = "A CLI tool for extracting data from DAT files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Extract {
        #[arg(help = "Path to DAT file (e.g., ./portal_1.dat)", short, long)]
        dat_path: String,
        #[arg(help = "Object ID to extract (e.g., 0321)")]
        object_id: String,
        #[arg(short, long, default_value = "./")]
        output_dir: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Extract {
            dat_path,
            object_id,
            output_dir,
        } => {
            println!(
                "Extract: {:?}, {:?}, {:?}!",
                dat_path, object_id, output_dir
            );
        }
    }

    Ok(())
}
