
pub(crate) use sqlx_core::database::{
    Database, HasArguments, HasStatement, HasStatementCache, HasValueRef,
};

use crate::{
    connection::RXQLiteConnection, /*RXQLiteArgumentValue,*/ RXQLiteArguments, RXQLiteColumn,
    /*RXQLiteConnection, */ RXQLiteQueryResult, RXQLiteRow, RXQLiteStatement,
    RXQLiteTransactionManager, RXQLiteTypeInfo, RXQLiteValue, RXQLiteValueRef,
};

/// RXQLite database driver.
#[derive(Debug)]
pub struct RXQLite;

impl Database for RXQLite {
    type Connection = RXQLiteConnection;

    type TransactionManager = RXQLiteTransactionManager;

    type Row = RXQLiteRow;

    type QueryResult = RXQLiteQueryResult;

    type Column = RXQLiteColumn;

    type TypeInfo = RXQLiteTypeInfo;

    type Value = RXQLiteValue;

    const NAME: &'static str = "RXQLite";

    const URL_SCHEMES: &'static [&'static str] = &["rxqlite"];
}

impl<'r> HasValueRef<'r> for RXQLite {
    type Database = RXQLite;

    type ValueRef = RXQLiteValueRef<'r>;
}

impl<'q> HasArguments<'q> for RXQLite {
    type Database = RXQLite;

    type Arguments = RXQLiteArguments;

    type ArgumentBuffer = Vec<rxqlite::Value>;
}

impl<'q> HasStatement<'q> for RXQLite {
    type Database = RXQLite;

    type Statement = RXQLiteStatement<'q>;
}

impl HasStatementCache for RXQLite {}
