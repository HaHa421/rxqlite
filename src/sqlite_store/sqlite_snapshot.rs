use sqlx::SqlitePool;
use serde::{Serialize, Deserialize};
use crate::sqlite_store::{SqliteAndPath,init_sqlite_connection};

use tokio::fs::File;
use tokio::io::{AsyncReadExt,AsyncWriteExt};


#[derive(Serialize, Deserialize,Default)]
pub struct SqliteSnaphot {
  db: Vec<u8>,
}
/*
PRAGMA schema.wal_checkpoint;
PRAGMA schema.wal_checkpoint(PASSIVE);
PRAGMA schema.wal_checkpoint(FULL);
PRAGMA schema.wal_checkpoint(RESTART);
PRAGMA schema.wal_checkpoint(TRUNCATE);

If the write-ahead log is enabled (via the journal_mode pragma), this pragma causes a checkpoint operation to run on database database, or on all attached databases if database is omitted. If write-ahead log mode is disabled, this pragma is a harmless no-op.

Invoking this pragma without an argument is equivalent to calling the sqlite3_wal_checkpoint() C interface.

Invoking this pragma with an argument is equivalent to calling the sqlite3_wal_checkpoint_v2() C interface with a 3rd parameter corresponding to the argument:
PASSIVE
Checkpoint as many frames as possible without waiting for any database readers or writers to finish. Sync the db file if all frames in the log are checkpointed. This mode is the same as calling the sqlite3_wal_checkpoint() C interface. The busy-handler callback is never invoked in this mode.
FULL
This mode blocks (invokes the busy-handler callback) until there is no database writer and all readers are reading from the most recent database snapshot. It then checkpoints all frames in the log file and syncs the database file. FULL blocks concurrent writers while it is running, but readers can proceed.
RESTART
This mode works the same way as FULL with the addition that after checkpointing the log file it blocks (calls the busy-handler callback) until all readers are finished with the log file. This ensures that the next client to write to the database file restarts the log file from the beginning. RESTART blocks concurrent writers while it is running, but allowed readers to proceed.
TRUNCATE
This mode works the same way as RESTART with the addition that the WAL file is truncated to zero bytes upon successful completion.
The wal_checkpoint pragma returns a single row with three integer columns. The first column is usually 0 but will be 1 if a RESTART or FULL or TRUNCATE checkpoint was blocked from completing, for example because another thread or process was actively using the database. In other words, the first column is 0 if the equivalent call to sqlite3_wal_checkpoint_v2() would have returned SQLITE_OK or 1 if the equivalent call would have returned SQLITE_BUSY. The second column is the number of modified pages that have been written to the write-ahead log file. The third column is the number of pages in the write-ahead log file that have been successfully moved back into the database file at the conclusion of the checkpoint. The second and third column are -1 if there is no write-ahead log, for example if this pragma is invoked on a database connection that is not in WAL mode.
*/
async fn flush_wal(pool: &SqlitePool) -> Result<(i64, i64, i64), sqlx::Error> {
    let row: (i64, i64, i64) = sqlx::query_as("PRAGMA wal_checkpoint;")
        .fetch_one(pool)
        .await?;

    Ok(row)
}

pub async fn make_snapshot(sqlite_and_path: &mut SqliteAndPath) -> Result<SqliteSnaphot,Box<dyn std::error::Error + Send + Sync + 'static>> {
  
  
  loop {
    let status = flush_wal(&sqlite_and_path.pool).await?;
    if status.0 == 0 {
      break;
    }
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
  }
  
  sqlite_and_path.pool.close().await;
  let mut sqlite_snapshot = SqliteSnaphot::default();
  {
    let mut file = File::open(&sqlite_and_path.path).await?;
    file.read_to_end(&mut sqlite_snapshot.db).await?;
    file.flush().await?;
  }
  sqlite_and_path.pool = 
    init_sqlite_connection(sqlite_and_path.path.to_str().unwrap()).await?;
  Ok(sqlite_snapshot)
}

pub async fn update_database_from_snapshot(sqlite_and_path: &mut SqliteAndPath,sqlite_snapshot: &SqliteSnaphot)->Result<(),Box<dyn std::error::Error + Send + Sync + 'static>> {
  loop {
    let status = flush_wal(&sqlite_and_path.pool).await?;
    if status.0 == 0 {
      break;
    }
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
  }
  sqlite_and_path.pool.close().await;
  tokio::fs::remove_file(&sqlite_and_path.path).await?;
  {
    let mut file = File::create(&sqlite_and_path.path).await?;
    file.write_all(&sqlite_snapshot.db).await?;
    file.flush().await?;
  }
  sqlite_and_path.pool = 
    init_sqlite_connection(sqlite_and_path.path.to_str().unwrap()).await?;
  Ok(())
}

