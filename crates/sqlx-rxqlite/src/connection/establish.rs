use super::*;

impl RXQLiteConnection {
    pub async fn establish(
        options: &RXQLiteConnectOptions,
    ) -> Result<Self, sqlx_core::error::Error> {
        let res = options.inner.connect().await;
        match res {
            Ok(conn) => Ok(Self { inner: conn }),
            Err(err) => Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{}", err).as_str(),
            ))),
        }
    }
}
