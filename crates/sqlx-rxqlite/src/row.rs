#![allow(clippy::rc_buffer)]

use std::sync::Arc;

use sqlx_core::column::ColumnIndex;
use sqlx_core::error::Error;
use sqlx_core::ext::ustr::UStr;
use sqlx_core::row::Row;
use sqlx_core::HashMap;

//use crate::statement::StatementHandle;
use crate::{RXQLite, RXQLiteColumn, RXQLiteValue, RXQLiteValueRef};

/// Implementation of [`Row`] for SQLite.
pub struct RXQLiteRow {
    pub(crate) values: Box<[RXQLiteValue]>,
    pub(crate) columns: Arc<Vec<RXQLiteColumn>>,
    pub(crate) column_names: Arc<HashMap<UStr, usize>>,
}

// Accessing values from the statement object is
// safe across threads as long as we don't call [sqlite3_step]

// we block ourselves from doing that by only exposing
// a set interface on [StatementHandle]

//unsafe impl Send for RXQLiteRow {}
//unsafe impl Sync for RXQLiteRow {}

impl RXQLiteRow {
    /*
    pub(crate) fn current(
        statement: &StatementHandle,
        columns: &Arc<Vec<RXQLiteColumn>>,
        column_names: &Arc<HashMap<UStr, usize>>,
    ) -> Self {
        let size = statement.column_count();
        let mut values = Vec::with_capacity(size);

        for i in 0..size {
            values.push(unsafe {
                let raw = statement.column_value(i);

                RXQLiteValue::new(raw, columns[i].type_info.clone())
            });
        }

        Self {
            values: values.into_boxed_slice(),
            columns: Arc::clone(columns),
            column_names: Arc::clone(column_names),
        }
    }
    */
}

impl Row for RXQLiteRow {
    type Database = RXQLite;

    fn columns(&self) -> &[RXQLiteColumn] {
        &self.columns
    }

    fn try_get_raw<I>(&self, index: I) -> Result<RXQLiteValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        Ok(RXQLiteValueRef::value(&self.values[index]))
    }
}

impl ColumnIndex<RXQLiteRow> for &'_ str {
    fn index(&self, row: &RXQLiteRow) -> Result<usize, Error> {
        row.column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}

// #[cfg(feature = "any")]
// impl From<RXQLiteRow> for crate::any::AnyRow {
//     #[inline]
//     fn from(row: RXQLiteRow) -> Self {
//         crate::any::AnyRow {
//             columns: row.columns.iter().map(|col| col.clone().into()).collect(),
//             kind: crate::any::row::AnyRowKind::RXQLite(row),
//         }
//     }
// }
