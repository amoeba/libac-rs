pub mod cli_helper;

use std::{error::Error, io::Cursor};

use crate::cli_helper::{find_file_by_id, index_dat};
use clap::{Parser, Subcommand};

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
            long("file")
        )]
        dat_file: String,
        #[arg(help = "Object ID to extract (e.g., 0321)")]
        object_id: String,
        #[arg(short, long, default_value = "./")]
        output_dir: String,
    },
    Read {
        #[arg(
            help = "Path or URI to DAT file (e.g., ./client_portal.dat)",
            short('f'),
            long("file")
        )]
        uri: String,
        #[arg(short('o'), long("offset"), help = "Object ID to extract (e.g., 0321)")]
        offset: String,
        #[arg(short('s'), long("size"))]
        file_size: String,
    },
}

#[cfg(feature = "tokio")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    use libac_rs::dat::{
        file_types::{dat_file::DatFile, texture::Texture},
        reader::file_reader::FileRangeReader,
    };

    let cli = Cli::parse();

    match cli.command {
        Commands::Extract {
            dat_file,
            object_id,
            output_dir,
        } => {
            use libac_rs::dat::reader::dat_file_reader::DatFileReader;

            println!(
                "cli::extract: {:?}, {:?}, {:?}!",
                dat_file, object_id, output_dir
            );

            let dat = index_dat(&dat_file).await?;
            let found_file = find_file_by_id(&dat, &object_id).await?;
            println!("Found file: {:?}", found_file);

            // Read the file into a buffer
            // TODO: This is messy

            // TODO: Can this setup be simplified?
            let file = tokio::fs::File::open(&dat_file).await?;
            let compat_file = tokio_util::compat::TokioAsyncReadCompatExt::compat(file);

            // My actual code
            let mut file_reader = FileRangeReader::new(compat_file);
            let mut reader = DatFileReader::new(
                found_file.file_size as usize,
                dat.header.block_size as usize,
            )?;
            let buf = reader
                .read_file(&mut file_reader, found_file.file_offset)
                .await
                .unwrap();

            // Step 3: Convert the buffer into our file
            // This is the common part
            let mut buf_reader = Cursor::new(buf);
            match found_file.file_type() {
                libac_rs::dat::enums::dat_file_type::DatFileType::Texture => {
                    let outer_file: DatFile<Texture> = DatFile::read(&mut buf_reader)?;
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
        Commands::Read {
            uri,
            offset,
            file_size,
        } => {
            println!("uri: {}, offset: {}, file_size: {}", uri, offset, file_size);
        }
    }

    Ok(())
}

#[cfg(not(feature = "tokio"))]
fn main() -> Result<(), Box<dyn Error>> {
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

            // TODO
            // 1. Determine method to use from uri
            // 2. Do the read
        }
    }

    Ok(())
}
