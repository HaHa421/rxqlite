use crate::{RXQLite, RXQLiteTypeInfo};
use sqlx_core::ext::ustr::UStr;

pub(crate) use sqlx_core::column::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct RXQLiteColumn {
    pub(crate) name: UStr,
    pub(crate) ordinal: usize,
    pub(crate) type_info: RXQLiteTypeInfo,
}

impl Column for RXQLiteColumn {
    type Database = RXQLite;

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn name(&self) -> &str {
        &*self.name
    }

    fn type_info(&self) -> &RXQLiteTypeInfo {
        &self.type_info
    }
}
