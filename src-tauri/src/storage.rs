use std::{error::Error, path::PathBuf};

fn get_data_dir()->Result<PathBuf,Box<dyn Error>>{
    let base_path =  dirs::data_dir().ok_or("Could not resolve base directory")?; //converting none
    //to an error
    let pulse_dir = base_path.join("pulse");
    std::fs::create_dir_all(&pulse_dir)?;
    std::fs::create_dir_all(pulse_dir.join("chunks"))?;
    Ok(pulse_dir) 
}

#[derive(Debug)]
pub struct TransferStore{
    conn: rusqlite::Connection
}

impl TransferStore {
    pub fn new()->Result<Self,Box<dyn Error>>{
        let pulse_dir = get_data_dir()?;
        let db_path = pulse_dir.join("pulse.db");
        let conn = rusqlite::Connection::open(&db_path)?;
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS transfers (
                transfer_id  TEXT PRIMARY KEY,
                file_hash    TEXT NOT NULL,
                filename     TEXT NOT NULL,
                file_size    INTEGER NOT NULL,
                total_chunks INTEGER NOT NULL DEFAULT 0,
                chunk_size   INTEGER NOT NULL DEFAULT 0,
                status       TEXT NOT NULL DEFAULT 'in_progress',
                role         TEXT NOT NULL,
                created_at   TEXT NOT NULL DEFAULT (datetime('now'))
            ); 
            CREATE TABLE IF NOT EXISTS chunks (
                transfer_id  TEXT NOT NULL,
                chunk_index  INTEGER NOT NULL,
                received_at  TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (transfer_id, chunk_index),
                FOREIGN KEY (transfer_id) REFERENCES transfers(transfer_id)
            );
            CREATE INDEX IF NOT EXISTS idx_transfers_file_hash ON transfers(file_hash);
        ")?;
        Ok(Self{conn})
    }
}
#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_successful_connection(){
        let store = TransferStore::new();
        assert!(store.is_ok());
        let store = store.unwrap();
        let table_count:i64 = store.conn
                .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('transfers', 'chunks')",
                [],
                |row| row.get(0),
                ).unwrap();
        assert_eq!(table_count,2);
    }
}
