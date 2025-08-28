mod db;
mod models;

use anyhow::Result;
use clap::Parser;
use db::Database;
use models::{FileMetadata, SpecificMetadata};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use id3::TagLike;
use mp4ameta;
use exif;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Index a directory
    Index {
        /// The directory to index
        #[arg(value_name = "DIRECTORY")]
        path: PathBuf,
        /// Path to the database file
        #[arg(short, long, default_value = "index.db")]
        db: String,
    },
    /// Search the index
    Search {
        /// The search query
        #[arg(value_name = "QUERY")]
        query: String,
        /// Path to the aabase file
        #[arg(short, long, default_value = "index.db")]
        db: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Index { path, db } => {
            let mut db = Database::new(db)?;
            index_directory(path, &mut db)?;
        }
        Commands::Search { query, db } => {
            let db = Database::new(db)?;
            let results = db.search_metadata(query)?;
            if results.is_empty() {
                println!("No results found for '{}'", query);
            } else {
                println!("Found {} results for '{}':", results.len(), query);
                for path in results {
                    println!("  - {}", path);
                }
            }
        }
    }
    Ok(())
}

fn index_directory(path: &PathBuf, db: &mut Database) -> Result<()> {
    let tx = db.transaction()?;
    println!("Indexing directory: {}", path.display());

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let file_path = entry.path();
            println!("Processing file: {}", file_path.display());

            let metadata = entry.metadata()?;

            let specific_metadata =
                if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
                    match ext.to_lowercase().as_str() {
                        "jpg" | "jpeg" | "png" => extract_image_metadata(file_path).ok(),
                        "mp3" => extract_mp3_metadata(file_path).ok(),
                        "flac" => extract_flac_metadata(file_path).ok(),
                        "mp4" => extract_mp4_metadata(file_path).ok(),
                        _ => None,
                    }
                } else {
                    None
                };

            let file_metadata = FileMetadata {
                path: file_path.to_string_lossy().to_string(),
                size: metadata.len(),
                modified: metadata.modified()?,
                specific: specific_metadata,
            };

            tx.write_metadata(&file_metadata)?;
        }
    }

    tx.commit()?;
    println!("Indexing complete.");
    Ok(())
}

fn extract_mp4_metadata(path: &Path) -> Result<SpecificMetadata> {
    let tag = mp4ameta::Tag::read_from_path(path)?;

    let mut extended_tags = BTreeMap::new();
    for (ident, data) in tag.data() {
        extended_tags.insert(ident.to_string(), vec![format!("{:?}", data)]);
    }

    Ok(SpecificMetadata::Video {
        artist: tag.artist().map(String::from),
        album: tag.album().map(String::from),
        title: tag.title().map(String::from),
        year: tag.year().and_then(|y| y.parse().ok()),
        extended_tags,
    })
}

fn extract_image_metadata(path: &Path) -> Result<SpecificMetadata> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let exif = exif::Reader::new().read_from_container(&mut buf_reader)?;

    let mut exif_map = BTreeMap::new();
    for f in exif.fields() {
        exif_map.insert(
            f.tag.to_string(),
            f.display_value().with_unit(&exif).to_string(),
        );
    }

    Ok(SpecificMetadata::Image { exif: exif_map })
}

fn extract_mp3_metadata(path: &Path) -> Result<SpecificMetadata> {
    let tag = id3::Tag::read_from_path(path)?;
    let extended_tags = tag
        .frames()
        .map(|frame| {
            (
                frame.id().to_string(),
                vec![frame.content().to_string()],
            )
        })
        .collect::<BTreeMap<_, _>>();

    Ok(SpecificMetadata::Audio {
        artist: tag.artist().map(String::from),
        album: tag.album().map(String::from),
        title: tag.title().map(String::from),
        year: tag.year(),
        extended_tags,
    })
}

fn extract_flac_metadata(path: &Path) -> Result<SpecificMetadata> {
    let tag = metaflac::Tag::read_from_path(path)?;
    let vorbis_comments = tag.vorbis_comments().ok_or_else(|| anyhow::anyhow!("No Vorbis comments found"))?;

    let extended_tags = vorbis_comments
        .comments
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    Ok(SpecificMetadata::Audio {
        artist: vorbis_comments.artist().map(|s| s.join(", ")),
        album: vorbis_comments.album().map(|s| s.join(", ")),
        title: vorbis_comments.title().map(|s| s.join(", ")),
        year: vorbis_comments.get("DATE").and_then(|d| d.get(0)).and_then(|y| y.parse().ok()),
        extended_tags,
    })
}
