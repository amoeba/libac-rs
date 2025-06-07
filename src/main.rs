pub mod cli_helper;

use std::{error::Error, io::Cursor, path::Path, ptr::read};

use clap::{Parser, Subcommand};
use libac_rs::dat::{
    file_types::{
        dat_file::{self, DatFile, DatFileRead},
        texture::{self, Texture},
    },
    reader::async_file_reader::{DatFileReader, FileRangeReader},
};
use tokio::fs::File;

use crate::cli_helper::{extract_texture_by_id, find_file_by_id, index_dat};

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
        #[arg(
            help = "Path to DAT file (e.g., ./client_portal.dat)",
            short('f'),
            long("dat_file")
        )]
        dat_file: String,
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
            dat_file,
            object_id,
            output_dir,
        } => {
            println!(
                "Extract: {:?}, {:?}, {:?}!",
                dat_file, object_id, output_dir
            );

            let dat = index_dat(&dat_file).await?;
            let found_file = find_file_by_id(&dat, &object_id).await?;

            println!("Found file: {:?}", dat_file);

            match found_file.file_type() {
                libac_rs::dat::enums::dat_file_type::DatFileType::Texture => {
                    let file = tokio::fs::File::open(&dat_file).await?;
                    let compat_file = tokio_util::compat::TokioAsyncReadCompatExt::compat(file);
                    let mut file_reader = FileRangeReader::new(compat_file);
                    let mut reader = DatFileReader::new(
                        found_file.file_size as usize,
                        dat.header.block_size as usize,
                    )?;
                    let result = reader
                        .read_file(&mut file_reader, found_file.file_offset)
                        .await
                        .unwrap();

                    let mut y = Cursor::new(result.buffer);
                    let outer_file: DatFile<Texture> = DatFile::read(&mut y)?;
                    let texture = outer_file.inner;
                    let output_path = format!("{}.png", object_id);
                    texture.to_png(&output_path, 1)?;
                    println!("Texture saved to {:?}", output_path);
                }
                _ => {
                    println!(
                        "Unsupported file type for extraction: {:?}",
                        found_file.file_type()
                    );
                }
            }
        }
    }

    Ok(())
}
