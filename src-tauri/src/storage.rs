use std::{error::Error, path::PathBuf};

fn get_data_dir()->Result<PathBuf,Box<dyn Error>>{
    let base_path =  dirs::data_dir().ok_or("Could not resolve base directory")?; //converting none
    //to an error
    let pulse_dir = base_path.join("pulse");
    std::fs::create_dir_all(&pulse_dir)?;
    std::fs::create_dir_all(pulse_dir.join("chunks"))?;
    Ok(pulse_dir) 
}

pub enum TransferStatus {
    InProgress,
    Complete,
    Failed 
}

impl TransferStatus {
    pub fn as_str(&self)-> &'static str {
        match self {
            TransferStatus::InProgress => "in_progress",
            TransferStatus::Complete => "complete",
            TransferStatus::Failed => "failed",
        }
    }
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
                status       TEXT NOT NULL DEFAULT 'in_progress' CHECK(status IN('in_progress','complete','failed')),
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

    pub fn create_transfer(&self, transfer_id: &str, file_hash: &str, filename: &str, file_size: u64,
        total_chunks: usize, chunk_size: usize, role: &str) -> Result<(), Box<dyn Error>>{
        self.conn.execute("
            INSERT INTO transfers (transfer_id,file_hash,filename,file_size,total_chunks,chunk_size,role) VALUES 
            (?1,?2,?3,?4,?5,?6,?7)
        ",
        (transfer_id,file_hash,filename,file_size as i64,total_chunks as i64,chunk_size as i64,role))?;
        Ok(())
    }
    pub fn mark_chunk_received(&self, transfer_id: &str,chunk_index: usize)->Result<(),Box<dyn Error>>{
        self.conn.execute("
            INSERT OR IGNORE INTO chunks (transfer_id,chunk_index) VALUES (?1,?2)
        ", (transfer_id,chunk_index as i64))?;
        Ok(())
    }

    pub fn get_received_chunks(&self, transfer_id: &str) -> Result<Vec<usize>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare("
            SELECT chunk_index FROM chunks WHERE transfer_id = ? ORDER BY chunk_index
        ")?;
        let chunks = stmt.query_map([transfer_id], |row| row.get::<_, i64>(0))?
        .collect::<Result<Vec<i64>, rusqlite::Error>>()?
        .into_iter()
        .map(|v| v as usize)
        .collect();
        Ok(chunks)
    }
    pub fn update_status(&self, transfer_id: &str, status: TransferStatus)->Result<(), Box<dyn Error>>{
        self.conn.execute("
            UPDATE transfers SET status = ? WHERE transfer_id = ?
        ", (status.as_str(),transfer_id))?;
        Ok(())
    }
}


#[cfg(test)]
mod transfer_store_tests{
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
    #[test]
    fn test_successful_crud(){
        let store = TransferStore::new();
        assert!(store.is_ok());
        let store = store.unwrap();
        assert!(store.create_transfer("abcd", "hash123", "demo.dat", 67, 32, 10, "receiver").is_ok());
        let (r1,r2,r3) = (
            store.mark_chunk_received("abcd", 0),
            store.mark_chunk_received("abcd",2),
            store.mark_chunk_received("abcd", 4)
        );
        assert!(r1.is_ok());
        assert!(r2.is_ok());
        assert!(r3.is_ok());

        let chunks = store.get_received_chunks("abcd");
        assert!(chunks.is_ok());
        let chunks = chunks.unwrap();
        assert_eq!(chunks,vec![0,2,4]);

        assert!(store.update_status("abcd", TransferStatus::Complete).is_ok());

        let status: String = store.conn.query_row(
            "SELECT status FROM transfers WHERE transfer_id = ?1",
            ["abcd"],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(status, "complete");

    }
}
