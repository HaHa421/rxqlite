
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::type_info::DataType;
use crate::types::Type;
use crate::{RXQLite, /*SqliteArgumentValue,*/ RaftSqliteTypeInfo, RaftSqliteValueRef};

impl Type<RXQLite> for f32 {
    fn type_info() -> RaftSqliteTypeInfo {
        RaftSqliteTypeInfo(DataType::Float)
    }
}

impl<'q> Encode<'q, RXQLite> for f32 {
    fn encode_by_ref(&self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::from(*self));

        IsNull::No
    }
}

impl<'r> Decode<'r, RXQLite> for f32 {
    fn decode(value: RaftSqliteValueRef<'r>) -> Result<f32, BoxDynError> {
        Ok(value.double()? as f32)
    }
}

impl Type<RXQLite> for f64 {
    fn type_info() -> RaftSqliteTypeInfo {
        RaftSqliteTypeInfo(DataType::Float)
    }
}

impl<'q> Encode<'q, RXQLite> for f64 {
    fn encode_by_ref(&self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::from(*self));

        IsNull::No
    }
}

impl<'r> Decode<'r, RXQLite> for f64 {
    fn decode(value: RaftSqliteValueRef<'r>) -> Result<f64, BoxDynError> {
        Ok(value.double()?)
    }
}
