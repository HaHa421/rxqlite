
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::type_info::DataType;
use crate::types::Type;
use crate::{RXQLite, /*RaftSqliteArgumentValue,*/ RaftSqliteTypeInfo, RaftSqliteValueRef};

impl Type<RXQLite> for u8 {
    fn type_info() -> RaftSqliteTypeInfo {
        RaftSqliteTypeInfo(DataType::Int)
    }

    fn compatible(ty: &RaftSqliteTypeInfo) -> bool {
        matches!(ty.0, DataType::Int | DataType::Int64)
    }
}

impl<'q> Encode<'q, RXQLite> for u8 {
    fn encode_by_ref(&self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::Int((*self).into()));

        IsNull::No
    }
}

impl<'r> Decode<'r, RXQLite> for u8 {
    fn decode(value: RaftSqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.int()? as _)
    }
}

impl Type<RXQLite> for u16 {
    fn type_info() -> RaftSqliteTypeInfo {
        RaftSqliteTypeInfo(DataType::Int)
    }

    fn compatible(ty: &RaftSqliteTypeInfo) -> bool {
        matches!(ty.0, DataType::Int | DataType::Int64)
    }
}

impl<'q> Encode<'q, RXQLite> for u16 {
    fn encode_by_ref(&self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::Int((*self).into()));

        IsNull::No
    }
}

impl<'r> Decode<'r, RXQLite> for u16 {
    fn decode(value: RaftSqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.int()? as _)
    }
}

impl Type<RXQLite> for u32 {
    fn type_info() -> RaftSqliteTypeInfo {
        RaftSqliteTypeInfo(DataType::Int64)
    }

    fn compatible(ty: &RaftSqliteTypeInfo) -> bool {
        matches!(ty.0, DataType::Int | DataType::Int64)
    }
}

impl<'q> Encode<'q, RXQLite> for u32 {
    fn encode_by_ref(&self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::Int((*self).into()));

        IsNull::No
    }
}

impl<'r> Decode<'r, RXQLite> for u32 {
    fn decode(value: RaftSqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.int64()? as _)
    }
}
