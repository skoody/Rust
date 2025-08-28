mod db;
mod models;
mod indexer;

use anyhow::Result;
use db::Database;
use slint::{Model, VecModel};
use std::rc::Rc;
use std::thread;
use rfd::FileDialog;

slint::include_modules!();

use models::{FileMetadata, SpecificMetadata};

fn format_metadata_details(metadata: &FileMetadata) -> String {
    let mut details = format!(
        "Path: {}\nSize: {} bytes\nModified: {}\n",
        metadata.path,
        metadata.size,
        humantime::format_rfc3339(metadata.modified).to_string()
    );

    if let Some(specific) = &metadata.specific {
        match specific {
            SpecificMetadata::Image { exif, png_chunks } => {
                details.push_str("\n--- Image Metadata ---\n");
                if !exif.is_empty() {
                    details.push_str("EXIF Data:\n");
                    for (tag, value) in exif {
                        details.push_str(&format!("  {}: {}\n", tag, value));
                    }
                }
                if let Some(chunks) = png_chunks {
                    if !chunks.is_empty() {
                        details.push_str("PNG Chunks:\n");
                        for (tag, value) in chunks {
                            details.push_str(&format!("  {}: {}\n", tag, value));
                        }
                    }
                }
            }
            SpecificMetadata::Audio { artist, album, title, year, extended_tags } => {
                details.push_str("\n--- Audio Metadata ---\n");
                if let Some(artist) = artist { details.push_str(&format!("Artist: {}\n", artist)); }
                if let Some(album) = album { details.push_str(&format!("Album: {}\n", album)); }
                if let Some(title) = title { details.push_str(&format!("Title: {}\n", title)); }
                if let Some(year) = year { details.push_str(&format!("Year: {}\n", year)); }
                if !extended_tags.is_empty() {
                    details.push_str("Extended Tags:\n");
                    for (tag, values) in extended_tags {
                        details.push_str(&format!("  {}: {:?}\n", tag, values));
                    }
                }
            }
            SpecificMetadata::Video { artist, album, title, year, extended_tags } => {
                details.push_str("\n--- Video Metadata ---\n");
                if let Some(artist) = artist { details.push_str(&format!("Artist: {}\n", artist)); }
                if let Some(album) = album { details.push_str(&format!("Album: {}\n", album)); }
                if let Some(title) = title { details.push_str(&format!("Title: {}\n", title)); }
                if let Some(year) = year { details.push_str(&format!("Year: {}\n", year)); }
                if !extended_tags.is_empty() {
                    details.push_str("Extended Tags:\n");
                    for (tag, values) in extended_tags {
                        details.push_str(&format!("  {}: {:?}\n", tag, values));
                    }
                }
            }
        }
    } else {
        details.push_str("\nNo specific metadata found.");
    }

    details
}


fn main() -> Result<()> {
    let app = AppWindow::new()?;

    let db = Rc::new(Database::new("index.db")?);

    let app_weak = app.as_weak();
    app.on_select_directory(move || {
        let app_weak = app_weak.clone();
        if let Some(path) = FileDialog::new().pick_folder() {
            thread::spawn(move || {
                // Create a new DB connection for this thread
                match Database::new("index.db") {
                    Ok(mut db_in_thread) => {
                        if let Err(e) = indexer::index_directory(&path, &mut db_in_thread) {
                            eprintln!("Error indexing directory: {}", e);
                        } else {
                            // Update UI from the thread
                            let _ = app_weak.upgrade_in_event_loop(|app| {
                                app.set_selected_file_details("Indexing complete.".into());
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to open database in thread: {}", e);
                    }
                }
            });
        }
    });

    let db_clone_for_search = db.clone();
    let app_weak = app.as_weak();
    app.on_search(move |query| {
        if let Some(app) = app_weak.upgrade() {
            let db = db_clone_for_search.clone();
            let results = db.search_metadata(&query).unwrap_or_else(|e| {
                eprintln!("Error searching metadata: {}", e);
                vec![]
            });

            let file_infos: Vec<FileInfo> = results
                .into_iter()
                .map(|metadata| FileInfo {
                    path: metadata.path.into(),
                    size: metadata.size.to_string().into(),
                })
                .collect();

            app.set_search_results(Rc::new(VecModel::from(file_infos)).into());
        }
    });

    let db_clone_for_select_file = db.clone();
    let app_weak = app.as_weak();
    app.on_file_selected(move |index| {
        if let Some(app) = app_weak.upgrade() {
            let db = db_clone_for_select_file.clone();
            if let Some(file_info) = app.get_search_results().row_data(index as usize) {
                if let Ok(metadata) = db.get_metadata_by_path(&file_info.path) {
                    let details = format_metadata_details(&metadata);
                    app.set_selected_file_details(details.into());
                } else {
                    app.set_selected_file_details("Could not retrieve details.".into());
                }
            }
        }
    });


    app.run()?;

    Ok(())
}
