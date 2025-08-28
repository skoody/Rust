use anyhow::Result;
use crate::db::Database;
use crate::models::{FileMetadata, SpecificMetadata};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use id3::TagLike;
use mp4ameta;
use exif;
use png;

pub fn index_directory(path: &PathBuf, db: &mut Database) -> Result<()> {
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
                        "jpg" | "jpeg" => extract_exif_metadata(file_path).ok(),
                        "png" => extract_png_metadata(file_path).ok(),
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

use png::Decoded;
use std::io::Read;

fn extract_png_metadata(path: &Path) -> Result<SpecificMetadata> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut decoder = png::StreamingDecoder::new();
    let mut chunks = BTreeMap::new();
    let mut data = Vec::new(); // Dummy vec for image data

    let mut offset = 0;
    while offset < buffer.len() {
        let (consumed, result) = decoder.update(&buffer[offset..], &mut data)?;
        offset += consumed;
        match result {
            Decoded::ChunkBegin(len, chunk_type) => {
                let key = std::str::from_utf8(&chunk_type.0)?.to_string();
                let value = format!("{} bytes", len);
                chunks.insert(key, value);
            }
            Decoded::ChunkComplete(_, chunk_type) => {
                if chunk_type.0 == *b"IEND" {
                    break;
                }
            }
            Decoded::ImageEnd => break,
            _ => (),
        }
    }

    Ok(SpecificMetadata::Image {
        exif: BTreeMap::new(), // No EXIF data for PNGs in this context
        png_chunks: Some(chunks),
    })
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

fn extract_exif_metadata(path: &Path) -> Result<SpecificMetadata> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let exif_data = exif::Reader::new().read_from_container(&mut buf_reader)?;

    let mut exif_map = BTreeMap::new();
    for f in exif_data.fields() {
        exif_map.insert(
            f.tag.to_string(),
            f.display_value().with_unit(&exif_data).to_string(),
        );
    }

    Ok(SpecificMetadata::Image {
        exif: exif_map,
        png_chunks: None,
    })
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
