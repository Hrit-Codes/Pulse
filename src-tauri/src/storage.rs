use std::{error::Error, path::PathBuf};
use uuid::Uuid;

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
                file_hash    TEXT DEFAULT NULL,
                filename     TEXT NOT NULL,
                file_size    INTEGER NOT NULL,
                total_chunks INTEGER NOT NULL DEFAULT 0,
                chunk_size   INTEGER NOT NULL DEFAULT 0,
                status       TEXT NOT NULL DEFAULT 'in_progress' CHECK(status IN('in_progress','complete','failed')),
                sender_id    TEXT NOT NULL,
                created_at   TEXT NOT NULL DEFAULT (datetime('now'))
            ); 
            CREATE TABLE IF NOT EXISTS chunks (
                transfer_id  TEXT NOT NULL,
                chunk_index  INTEGER NOT NULL,
                received_at  TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (transfer_id, chunk_index),
                FOREIGN KEY (transfer_id) REFERENCES transfers(transfer_id)
            );
            CREATE TABLE IF NOT EXISTS device_identity (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS sent_transfers (
                transfer_id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_transfers_file_hash ON transfers(file_hash);
        ")?;
        Ok(Self{conn})
    }
    
    pub fn get_or_create_identity(&self)->Result<(String,String),Box<dyn Error>>{
        let result: rusqlite::Result<(String, String)> = self.conn.query_row(
            "SELECT id, name FROM device_identity LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );

        match result {
            Ok((id, name)) => Ok((id, name)),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                let id = Uuid::new_v4().to_string();
                let name = "Rochak's Macbook".to_string();

                self.conn.execute(
                    "INSERT INTO device_identity (id, name) VALUES (?1, ?2)",
                    (&id, &name),
                )?;

                Ok((id, name))
            }
            Err(e) => Err(e.into()),
        } 
    }

    pub fn get_chunk_dir(&self)->Result<PathBuf,Box<dyn Error>>{
        let pulse_dir = get_data_dir()?;
        let chunk_dir = pulse_dir.join("chunks");
        Ok(chunk_dir)
    }
    
    pub fn record_sent_transfer(&self,transfer_id: &str,file_path: &str)->Result<(),Box<dyn Error>>{
        self.conn.execute("
            INSERT INTO sent_transfers (transfer_id,file_path) VALUES 
            (?1,?2)
        ",
        (transfer_id,file_path))?;
        Ok(())
    }
    pub fn get_sent_transfer_path(&self,transfer_id: &str)->Result<Option<String>,Box<dyn Error>>{
        let result: rusqlite::Result<String> = self.conn.query_row(
            "SELECT file_path FROM sent_transfers WHERE transfer_id = ?1",
            [transfer_id],
            |row| row.get(0),
        );

        match result {
            Ok(file_path) => Ok(Some(file_path)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_transfer(&self, transfer_id: &str, filename: &str, file_size: u64,
        total_chunks: usize, chunk_size: usize, sender_id: &str) -> Result<(), Box<dyn Error>>{
        self.conn.execute("
            INSERT INTO transfers (transfer_id,filename,file_size,total_chunks,chunk_size,sender_id) VALUES 
            (?1,?2,?3,?4,?5,?6)
        ",
        (transfer_id,filename,file_size as i64,total_chunks as i64,chunk_size as i64,sender_id))?;
        Ok(())
    }

    pub fn mark_chunks_received(&self,transfer_id: &str,chunk_indices: &[usize]) -> Result<(), Box<dyn Error>> {
        if chunk_indices.is_empty() {
            return Ok(());
        }

        let placeholders = std::iter::repeat("(?, ?)")
            .take(chunk_indices.len())
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "INSERT OR IGNORE INTO chunks (transfer_id, chunk_index)
            VALUES {}",
            placeholders
        );
        let tx = self.conn.unchecked_transaction()?;

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::with_capacity(chunk_indices.len() * 2);

        for &index in chunk_indices {
            params.push(Box::new(transfer_id));
            params.push(Box::new(index as i64));
        }
        tx.execute(&sql, rusqlite::params_from_iter(params.iter()))?;
        tx.commit()?;
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
    pub fn get_in_progress_transfers(&self) -> Result<Vec<(String, String, String, i64,i64)>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT transfer_id, sender_id, filename, file_size,total_chunks FROM transfers WHERE status = 'in_progress'"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        Ok(rows)
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
        assert!(store.create_transfer("abcd", "demo.dat", 67, 32, 10, "sender_id").is_ok());
        let r1 = store.mark_chunks_received("abcd", &[0,2,4]);
        assert!(r1.is_ok());

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
        let sender_id: String = store.conn.query_row(
            "SELECT sender_id FROM transfers WHERE transfer_id = ?1",
            ["abcd"],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(sender_id, "sender_id");

    }
    #[test]
    fn test_successful_record(){
        let store = TransferStore::new();
        assert!(store.is_ok());
        let store = store.unwrap();
        assert!(store.record_sent_transfer("demo_id", "home/path/transfer").is_ok());
        let path: String = store.conn.query_row(
            "SELECT file_path FROM sent_transfers WHERE transfer_id = ?1",
            ["demo_id"],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(path,"home/path/transfer");

    }
    #[test]
    fn test_transfer_list(){
        let store = TransferStore::new();
        assert!(store.is_ok());
        let store = store.unwrap();
        assert!(store.create_transfer("id", "filename", 8, 4 ,3, "sender_id").is_ok());
        let list = store.get_in_progress_transfers();
        assert!(list.is_ok());
        let list = list.unwrap();
        assert_eq!(("id".to_string(),"sender_id".to_string(),"filename".to_string(),8),list[0]);
    }
}
