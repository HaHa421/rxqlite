use crate::error::RaftSqliteError;
use crate::type_info::DataType;
use crate::RaftSqliteColumn;
use crate::{
    RXQLite, RaftSqliteConnection, RaftSqliteQueryResult, RaftSqliteRow, RaftSqliteStatement, RaftSqliteTypeInfo,
    RaftSqliteValue,
};
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::TryStreamExt;
use sqlx_core::describe::Describe;
use sqlx_core::error::Error;
use sqlx_core::executor::{Execute, Executor};
use sqlx_core::ext::ustr::UStr;
use sqlx_core::try_stream;
use sqlx_core::Either;

impl<'c> Executor<'c> for &'c mut RaftSqliteConnection {
    type Database = RXQLite;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<RaftSqliteQueryResult, RaftSqliteRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();
        //let persistent = query.persistent() && arguments.is_some();

        //let args = Vec::with_capacity(arguments.len());

        Box::pin(try_stream! {
          let rows = self.inner.execute(sql, match arguments {
            Some(arguments)=>arguments.values,
            _=>vec![],
          }).await;
          match rows {
            Ok(rows)=> {
              //println!("{}:({})",file!(),line!());

              //pin_mut!(cursor);
              let mut rows_iter=rows.into_iter();
              while let Some(row) = rows_iter.next() {
                let size = row.inner.len();
                let mut values = Vec::with_capacity(size);
                let mut columns = Vec::with_capacity(size);
                //let mut column_names = Vec::with_capacity(size);
                  for (i,value) in row.inner.into_iter().enumerate() {
                    values.push(RaftSqliteValue::new(value,RaftSqliteTypeInfo(DataType::Null)));
                    columns.push(RaftSqliteColumn{
                      name : UStr::from(""),
                      ordinal: i,
                      type_info: RaftSqliteTypeInfo(DataType::Null),
                    });
                  }
                  let row=Either::Right(RaftSqliteRow {
                    values: values.into_boxed_slice(),
                    columns: columns.into(),
                    column_names : Default::default(),
                  });
                  r#yield!(row);
              }
              Ok(())
            }
            Err(err)=> {
              Err(RaftSqliteError{
                inner: err,
              }.into())
            }
          }
        })
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<RaftSqliteRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let mut s = self.fetch_many(query);

        Box::pin(async move {
            while let Some(v) = s.try_next().await? {
                if let Either::Right(r) = v {
                    return Ok(Some(r));
                }
            }

            Ok(None)
        })
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        _sql: &'q str,
        _parameters: &[RaftSqliteTypeInfo],
    ) -> BoxFuture<'e, Result<RaftSqliteStatement<'q>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "prepare_with not supported",
            )))
        })
    }
    #[doc(hidden)]
    fn describe<'e, 'q: 'e>(self, _sql: &'q str) -> BoxFuture<'e, Result<Describe<RXQLite>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "describe not supported",
            )))
        })
    }
}
