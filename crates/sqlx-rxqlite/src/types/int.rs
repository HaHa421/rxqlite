
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::type_info::DataType;
use crate::types::Type;
use crate::{RXQLite, /*RXQLiteArgumentValue,*/ RXQLiteTypeInfo, RXQLiteValueRef};

impl Type<RXQLite> for i8 {
    fn type_info() -> RXQLiteTypeInfo {
        RXQLiteTypeInfo(DataType::Int)
    }

    fn compatible(ty: &RXQLiteTypeInfo) -> bool {
        matches!(ty.0, DataType::Int | DataType::Int64)
    }
}

impl<'q> Encode<'q, RXQLite> for i8 {
    fn encode_by_ref(&self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::Int((*self).into()));

        IsNull::No
    }
}

impl<'r> Decode<'r, RXQLite> for i8 {
    fn decode(value: RXQLiteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.int()?.try_into()?)
    }
}

impl Type<RXQLite> for i16 {
    fn type_info() -> RXQLiteTypeInfo {
        RXQLiteTypeInfo(DataType::Int)
    }

    fn compatible(ty: &RXQLiteTypeInfo) -> bool {
        matches!(ty.0, DataType::Int | DataType::Int64)
    }
}

impl<'q> Encode<'q, RXQLite> for i16 {
    fn encode_by_ref(&self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::Int((*self).into()));

        IsNull::No
    }
}

impl<'r> Decode<'r, RXQLite> for i16 {
    fn decode(value: RXQLiteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.int()?.try_into()?)
    }
}

impl Type<RXQLite> for i32 {
    fn type_info() -> RXQLiteTypeInfo {
        RXQLiteTypeInfo(DataType::Int)
    }

    fn compatible(ty: &RXQLiteTypeInfo) -> bool {
        matches!(ty.0, DataType::Int | DataType::Int64)
    }
}

impl<'q> Encode<'q, RXQLite> for i32 {
    fn encode_by_ref(&self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::Int((*self).into()));

        IsNull::No
    }
}

impl<'r> Decode<'r, RXQLite> for i32 {
    fn decode(value: RXQLiteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.int()?)
    }
}

impl Type<RXQLite> for i64 {
    fn type_info() -> RXQLiteTypeInfo {
        RXQLiteTypeInfo(DataType::Int64)
    }

    fn compatible(ty: &RXQLiteTypeInfo) -> bool {
        matches!(ty.0, DataType::Int | DataType::Int64)
    }
}

impl<'q> Encode<'q, RXQLite> for i64 {
    fn encode_by_ref(&self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::Int((*self).into()));

        IsNull::No
    }
}

impl<'r> Decode<'r, RXQLite> for i64 {
    fn decode(value: RXQLiteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.int64()?)
    }
}
