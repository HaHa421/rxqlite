use crate::{RXQLite, RaftSqliteTypeInfo};
use sqlx_core::ext::ustr::UStr;

pub(crate) use sqlx_core::column::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct RaftSqliteColumn {
    pub(crate) name: UStr,
    pub(crate) ordinal: usize,
    pub(crate) type_info: RaftSqliteTypeInfo,
}

impl Column for RaftSqliteColumn {
    type Database = RXQLite;

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn name(&self) -> &str {
        &*self.name
    }

    fn type_info(&self) -> &RaftSqliteTypeInfo {
        &self.type_info
    }
}
