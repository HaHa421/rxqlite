
pub(crate) use sqlx_core::database::{
    Database, HasArguments, HasStatement, HasStatementCache, HasValueRef,
};

use crate::{
    connection::RaftSqliteConnection, /*RaftSqliteArgumentValue,*/ RaftSqliteArguments, RaftSqliteColumn,
    /*RaftSqliteConnection, */ RaftSqliteQueryResult, RaftSqliteRow, RaftSqliteStatement,
    RaftSqliteTransactionManager, RaftSqliteTypeInfo, RaftSqliteValue, RaftSqliteValueRef,
};

/// RXQLite database driver.
#[derive(Debug)]
pub struct RXQLite;

impl Database for RXQLite {
    type Connection = RaftSqliteConnection;

    type TransactionManager = RaftSqliteTransactionManager;

    type Row = RaftSqliteRow;

    type QueryResult = RaftSqliteQueryResult;

    type Column = RaftSqliteColumn;

    type TypeInfo = RaftSqliteTypeInfo;

    type Value = RaftSqliteValue;

    const NAME: &'static str = "RXQLite";

    const URL_SCHEMES: &'static [&'static str] = &["rxqlite"];
}

impl<'r> HasValueRef<'r> for RXQLite {
    type Database = RXQLite;

    type ValueRef = RaftSqliteValueRef<'r>;
}

impl<'q> HasArguments<'q> for RXQLite {
    type Database = RXQLite;

    type Arguments = RaftSqliteArguments;

    type ArgumentBuffer = Vec<rxqlite::Value>;
}

impl<'q> HasStatement<'q> for RXQLite {
    type Database = RXQLite;

    type Statement = RaftSqliteStatement<'q>;
}

impl HasStatementCache for RXQLite {}
