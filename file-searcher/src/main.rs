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
                    let details = format!("{:#?}", metadata);
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
