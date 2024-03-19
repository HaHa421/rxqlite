#![deny(unused_extern_crates)]
#![deny(warnings)]
#![allow(deprecated)]

#[macro_use]
extern crate sqlx_core;

pub(crate) use sqlx_core::driver_prelude::*;

pub mod error;
pub mod type_info;
use type_info::RaftSqliteTypeInfo;

mod types;

mod options;
pub use options::RaftSqliteConnectOptions;
pub mod connection;
use connection::RaftSqliteConnection;
pub mod arguments;
use arguments::RaftSqliteArguments;
pub mod column;
use column::RaftSqliteColumn;

pub mod statement;
use statement::RaftSqliteStatement;

pub mod row;
use row::RaftSqliteRow;

pub mod query_result;
use query_result::RaftSqliteQueryResult;

pub mod transaction;
use transaction::RaftSqliteTransactionManager;
pub mod database;
use database::RXQLite;

pub mod value;
use value::*;

impl_into_arguments_for_arguments!(RaftSqliteArguments);
impl_acquire!(RXQLite, RaftSqliteConnection);
impl_column_index_for_row!(RaftSqliteRow);
impl_column_index_for_statement!(RaftSqliteStatement);

pub type RaftSqlitePool = crate::pool::Pool<RXQLite>;

/// An alias for [`PoolOptions`][crate::pool::PoolOptions], specialized for SQLite.
pub type RaftSqlitePoolOptions = crate::pool::PoolOptions<RXQLite>;
