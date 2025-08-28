use crate::models::FileMetadata;
use anyhow::Result;
use rusqlite::{Connection, Transaction};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                path     TEXT PRIMARY KEY,
                metadata TEXT NOT NULL
            )",
            [],
        )?;
        Ok(Database { conn })
    }

    pub fn transaction(&mut self) -> Result<DbTransaction> {
        let tx = self.conn.transaction()?;
        Ok(DbTransaction { tx })
    }

    pub fn search_metadata(&self, query: &str) -> Result<Vec<FileMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT metadata FROM files WHERE metadata LIKE ?",
        )?;
        let mut rows = stmt.query([format!("%{}%", query)])?;

        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            let metadata_json: String = row.get(0)?;
            let metadata: FileMetadata = serde_json::from_str(&metadata_json)?;
            results.push(metadata);
        }

        Ok(results)
    }

    pub fn get_metadata_by_path(&self, path: &str) -> Result<FileMetadata> {
        let mut stmt = self.conn.prepare(
            "SELECT metadata FROM files WHERE path = ?",
        )?;
        let mut rows = stmt.query([path])?;

        if let Some(row) = rows.next()? {
            let metadata_json: String = row.get(0)?;
            let metadata: FileMetadata = serde_json::from_str(&metadata_json)?;
            Ok(metadata)
        } else {
            Err(anyhow::anyhow!("File not found in database"))
        }
    }
}

pub struct DbTransaction<'a> {
    tx: Transaction<'a>,
}

impl<'a> DbTransaction<'a> {
    pub fn write_metadata(&self, metadata: &FileMetadata) -> Result<()> {
        let json = serde_json::to_string(metadata)?;
        self.tx.execute(
            "INSERT OR REPLACE INTO files (path, metadata) VALUES (?1, ?2)",
            (&metadata.path, &json),
        )?;
        Ok(())
    }

    pub fn commit(self) -> Result<()> {
        self.tx.commit()?;
        Ok(())
    }
}
