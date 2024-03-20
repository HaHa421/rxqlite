use futures_core::future::BoxFuture;

use crate::{RXQLite, RXQLiteConnection};
use sqlx_core::error::Error;
use sqlx_core::transaction::TransactionManager;

/// Implementation of [`TransactionManager`] for SQLite.
pub struct RXQLiteTransactionManager;

impl TransactionManager for RXQLiteTransactionManager {
    type Database = RXQLite;

    fn begin(_conn: &mut RXQLiteConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "begin not supported",
            )))
        })
    }

    fn commit(_conn: &mut RXQLiteConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "commit not supported",
            )))
        })
    }

    fn rollback(_conn: &mut RXQLiteConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "rollback not supported",
            )))
        })
    }

    fn start_rollback(_conn: &mut RXQLiteConnection) {
        //conn.worker.start_rollback().ok();
    }
}
