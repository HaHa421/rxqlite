use futures_core::future::BoxFuture;

use crate::{RXQLite, RaftSqliteConnection};
use sqlx_core::error::Error;
use sqlx_core::transaction::TransactionManager;

/// Implementation of [`TransactionManager`] for SQLite.
pub struct RaftSqliteTransactionManager;

impl TransactionManager for RaftSqliteTransactionManager {
    type Database = RXQLite;

    fn begin(_conn: &mut RaftSqliteConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "begin not supported",
            )))
        })
    }

    fn commit(_conn: &mut RaftSqliteConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "commit not supported",
            )))
        })
    }

    fn rollback(_conn: &mut RaftSqliteConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "rollback not supported",
            )))
        })
    }

    fn start_rollback(_conn: &mut RaftSqliteConnection) {
        //conn.worker.start_rollback().ok();
    }
}
