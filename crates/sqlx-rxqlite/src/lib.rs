#![deny(unused_extern_crates)]
#![deny(warnings)]
#![allow(deprecated)]

#[macro_use]
extern crate sqlx_core;

pub(crate) use sqlx_core::driver_prelude::*;

pub mod error;
pub mod type_info;
use type_info::RXQLiteTypeInfo;

mod types;

mod options;
pub use options::RXQLiteConnectOptions;
pub mod connection;
use connection::RXQLiteConnection;
pub mod arguments;
use arguments::RXQLiteArguments;
pub mod column;
use column::RXQLiteColumn;

pub mod statement;
use statement::RXQLiteStatement;

pub mod row;
use row::RXQLiteRow;

pub mod query_result;
use query_result::RXQLiteQueryResult;

pub mod transaction;
use transaction::RXQLiteTransactionManager;
pub mod database;
use database::RXQLite;

pub mod value;
use value::*;

impl_into_arguments_for_arguments!(RXQLiteArguments);
impl_acquire!(RXQLite, RXQLiteConnection);
impl_column_index_for_row!(RXQLiteRow);
impl_column_index_for_statement!(RXQLiteStatement);

pub type RXQLitePool = crate::pool::Pool<RXQLite>;

/// An alias for [`PoolOptions`][crate::pool::PoolOptions], specialized for SQLite.
pub type RXQLitePoolOptions = crate::pool::PoolOptions<RXQLite>;
