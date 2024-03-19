
use crate::connection::RaftSqliteConnection;
use crate::options::RaftSqliteConnectOptions;

use futures_core::future::BoxFuture;
use log::LevelFilter;
use sqlx_core::connection::ConnectOptions;
use sqlx_core::error::Error;

use std::str::FromStr;
use std::time::Duration;
use url::Url;

impl ConnectOptions for RaftSqliteConnectOptions {
    type Connection = RaftSqliteConnection;
    // borrowed from sqlx-mysql
    fn from_url(url: &Url) -> Result<Self, Error> {
        let mut options = rxqlite::ConnectOptions::default();
        if let Some(host) = url.host_str() {
            options.leader_host = host.into();
        }

        if let Some(port) = url.port() {
            options.leader_port = port;
        }
        /*
        let username = url.username();
        if !username.is_empty() {
            options.user = Some(
                percent_decode_str(username)
                    .decode_utf8()
                    .map_err(Error::config)?
                    .to_string(),
            );
        }

        if let Some(password) = url.password() {
            options.pass = Some(
                percent_decode_str(password)
                    .decode_utf8()
                    .map_err(Error::config)?
                    .to_string(),
            );
        }
        */
        /*
        let path = url.path().trim_start_matches('/');
        if !path.is_empty() {
            options = options.database(path);
        }
        */
        for (key, value) in url.query_pairs().into_iter() {
            match &*key {
              "ssl"=> {
                if value == "yes" || value == "1" {
                  options.scheme=rxqlite::Scheme::HTTPS;
                }
              }
              "ssl-insecure"=> {
                if value == "yes" || value == "1" {
                  options.scheme=rxqlite::Scheme::HTTPS;
                  options.accept_invalid_cert=true;
                }
              }
              _=>{}
            }
        }
        Ok(Self { inner: options })
    }
    fn to_url_lossy(&self) -> Url {
        self.build_url()
    }

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, Error>>
    where
        Self::Connection: Sized,
    {
        Box::pin(async move {
            let conn = RaftSqliteConnection::establish(self).await?;
            /*
                        // After the connection is established, we initialize by configuring a few
                        // connection parameters

                        // https://mariadb.com/kb/en/sql-mode/

                        // PIPES_AS_CONCAT - Allows using the pipe character (ASCII 124) as string concatenation operator.
                        //                   This means that "A" || "B" can be used in place of CONCAT("A", "B").

                        // NO_ENGINE_SUBSTITUTION - If not set, if the available storage engine specified by a CREATE TABLE is
                        //                          not available, a warning is given and the default storage
                        //                          engine is used instead.

                        // NO_ZERO_DATE - Don't allow '0000-00-00'. This is invalid in Rust.

                        // NO_ZERO_IN_DATE - Don't allow 'YYYY-00-00'. This is invalid in Rust.

                        // --

                        // Setting the time zone allows us to assume that the output
                        // from a TIMESTAMP field is UTC

                        // --

                        // https://mathiasbynens.be/notes/mysql-utf8mb4

                        let mut sql_mode = Vec::new();
                        if self.pipes_as_concat {
                            sql_mode.push(r#"PIPES_AS_CONCAT"#);
                        }
                        if self.no_engine_subsitution {
                            sql_mode.push(r#"NO_ENGINE_SUBSTITUTION"#);
                        }

                        let mut options = Vec::new();
                        if !sql_mode.is_empty() {
                            options.push(format!(
                                r#"sql_mode=(SELECT CONCAT(@@sql_mode, ',{}'))"#,
                                sql_mode.join(",")
                            ));
                        }
                        if let Some(timezone) = &self.timezone {
                            options.push(format!(r#"time_zone='{}'"#, timezone));
                        }
                        if self.set_names {
                            options.push(format!(
                                r#"NAMES {} COLLATE {}"#,
                                conn.stream.charset.as_str(),
                                conn.stream.collation.as_str()
                            ))
                        }

                        if !options.is_empty() {
                            conn.execute(&*format!(r#"SET {};"#, options.join(",")))
                                .await?;
                        }
            */
            Ok(conn)
        })
    }

    fn log_statements(self, _level: LevelFilter) -> Self {
        //self.log_settings.log_statements(level);
        self
    }

    fn log_slow_statements(self, _level: LevelFilter, _duration: Duration) -> Self {
        //self.log_settings.log_slow_statements(level, duration);
        self
    }
}

impl FromStr for RaftSqliteConnectOptions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let url: Url = s.parse().map_err(Error::config)?;
        Self::from_url(&url)
    }
}
