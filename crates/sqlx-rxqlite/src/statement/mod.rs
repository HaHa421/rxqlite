use crate::column::ColumnIndex;
use crate::error::Error;
use crate::{RXQLite, RXQLiteArguments, RXQLiteColumn, RXQLiteTypeInfo};
use sqlx_core::ext::ustr::UStr;
use sqlx_core::{Either, HashMap};
use std::borrow::Cow;
use std::sync::Arc;

use sqlx_core::impl_statement_query;
pub(crate) use sqlx_core::statement::*;
/*
mod handle;
pub(super) mod unlock_notify;
mod r#virtual;

pub(crate) use handle::StatementHandle;
pub(crate) use r#virtual::VirtualStatement;
*/
#[derive(Debug, Clone)]
#[allow(clippy::rc_buffer)]
pub struct RXQLiteStatement<'q> {
    pub(crate) sql: Cow<'q, str>,
    pub(crate) parameters: usize,
    pub(crate) columns: Arc<Vec<RXQLiteColumn>>,
    pub(crate) column_names: Arc<HashMap<UStr, usize>>,
}

impl<'q> Statement<'q> for RXQLiteStatement<'q> {
    type Database = RXQLite;

    fn to_owned(&self) -> RXQLiteStatement<'static> {
        RXQLiteStatement::<'static> {
            sql: Cow::Owned(self.sql.clone().into_owned()),
            parameters: self.parameters,
            columns: Arc::clone(&self.columns),
            column_names: Arc::clone(&self.column_names),
        }
    }

    fn sql(&self) -> &str {
        &self.sql
    }

    fn parameters(&self) -> Option<Either<&[RXQLiteTypeInfo], usize>> {
        Some(Either::Right(self.parameters))
    }

    fn columns(&self) -> &[RXQLiteColumn] {
        &self.columns
    }

    impl_statement_query!(RXQLiteArguments);
}

impl ColumnIndex<RXQLiteStatement<'_>> for &'_ str {
    fn index(&self, statement: &RXQLiteStatement<'_>) -> Result<usize, Error> {
        statement
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}

// #[cfg(feature = "any")]
// impl<'q> From<RXQLiteStatement<'q>> for crate::any::AnyStatement<'q> {
//     #[inline]
//     fn from(statement: RXQLiteStatement<'q>) -> Self {
//         crate::any::AnyStatement::<'q> {
//             columns: statement
//                 .columns
//                 .iter()
//                 .map(|col| col.clone().into())
//                 .collect(),
//             column_names: statement.column_names,
//             parameters: Some(Either::Right(statement.parameters)),
//             sql: statement.sql,
//         }
//     }
// }
