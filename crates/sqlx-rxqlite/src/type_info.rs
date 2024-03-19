use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

//use libsqlite3_sys::{SQLITE_BLOB, SQLITE_FLOAT, SQLITE_INTEGER, SQLITE_NULL, SQLITE_TEXT};

use crate::error::BoxDynError;

pub(crate) use sqlx_core::type_info::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum DataType {
    Null,
    Int,
    Float,
    Text,
    Blob,

    // TODO: Support NUMERIC
    #[allow(dead_code)]
    Numeric,

    // non-standard extensions
    Bool,
    Int64,
    Date,
    Time,
    Datetime,
}

/// Type information for a SQLite type.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct RXQLiteTypeInfo(pub(crate) DataType);

impl Display for RXQLiteTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(self.name())
    }
}

impl TypeInfo for RXQLiteTypeInfo {
    fn is_null(&self) -> bool {
        matches!(self.0, DataType::Null)
    }

    fn name(&self) -> &str {
        match self.0 {
            DataType::Null => "NULL",
            DataType::Text => "TEXT",
            DataType::Float => "REAL",
            DataType::Blob => "BLOB",
            DataType::Int | DataType::Int64 => "INTEGER",
            DataType::Numeric => "NUMERIC",

            // non-standard extensions
            DataType::Bool => "BOOLEAN",
            DataType::Date => "DATE",
            DataType::Time => "TIME",
            DataType::Datetime => "DATETIME",
        }
    }
}

// note: this implementation is particularly important as this is how the macros determine
//       what Rust type maps to what *declared* SQL type
// <https://www.sqlite.org/datatype3.html#affname>
impl FromStr for DataType {
    type Err = BoxDynError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        Ok(match &*s {
            "int4" => DataType::Int,
            "int8" => DataType::Int64,
            "boolean" | "bool" => DataType::Bool,

            "date" => DataType::Date,
            "time" => DataType::Time,
            "datetime" | "timestamp" => DataType::Datetime,

            _ if s.contains("int") => DataType::Int64,

            _ if s.contains("char") || s.contains("clob") || s.contains("text") => DataType::Text,

            _ if s.contains("blob") => DataType::Blob,

            _ if s.contains("real") || s.contains("floa") || s.contains("doub") => DataType::Float,

            _ => {
                return Err(format!("unknown type: `{s}`").into());
            }
        })
    }
}

// #[cfg(feature = "any")]
// impl From<RXQLiteTypeInfo> for crate::any::AnyTypeInfo {
//     #[inline]
//     fn from(ty: RXQLiteTypeInfo) -> Self {
//         crate::any::AnyTypeInfo(crate::any::type_info::AnyTypeInfoKind::RXQLite(ty))
//     }
// }

#[test]
fn test_data_type_from_str() -> Result<(), BoxDynError> {
    assert_eq!(DataType::Int, "INT4".parse()?);

    assert_eq!(DataType::Int64, "INT".parse()?);
    assert_eq!(DataType::Int64, "INTEGER".parse()?);
    assert_eq!(DataType::Int64, "INTBIG".parse()?);
    assert_eq!(DataType::Int64, "MEDIUMINT".parse()?);

    assert_eq!(DataType::Int64, "BIGINT".parse()?);
    assert_eq!(DataType::Int64, "UNSIGNED BIG INT".parse()?);
    assert_eq!(DataType::Int64, "INT8".parse()?);

    assert_eq!(DataType::Text, "CHARACTER(20)".parse()?);
    assert_eq!(DataType::Text, "NCHAR(55)".parse()?);
    assert_eq!(DataType::Text, "TEXT".parse()?);
    assert_eq!(DataType::Text, "CLOB".parse()?);

    assert_eq!(DataType::Blob, "BLOB".parse()?);

    assert_eq!(DataType::Float, "REAL".parse()?);
    assert_eq!(DataType::Float, "FLOAT".parse()?);
    assert_eq!(DataType::Float, "DOUBLE PRECISION".parse()?);

    assert_eq!(DataType::Bool, "BOOLEAN".parse()?);
    assert_eq!(DataType::Bool, "BOOL".parse()?);

    assert_eq!(DataType::Datetime, "DATETIME".parse()?);
    assert_eq!(DataType::Time, "TIME".parse()?);
    assert_eq!(DataType::Date, "DATE".parse()?);

    Ok(())
}
