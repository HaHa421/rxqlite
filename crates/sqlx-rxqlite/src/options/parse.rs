
use crate::RXQLiteConnectOptions;
use sqlx_core::Url;

impl RXQLiteConnectOptions {
    pub(crate) fn build_url(&self) -> Url {
        let mut url = Url::parse(&format!(
            "rxqlite://{}:{}",
            /*self.inner.username, */ self.inner.leader_host, self.inner.leader_port
        ))
        .expect("BUG: generated un-parseable URL");
        /*
        if let Some(user) = &self.inner.user {
            let _ = url.set_username(&user);
        }

        if let Some(password) = &self.inner.pass {
            let password = utf8_percent_encode(&password, NON_ALPHANUMERIC).to_string();
            let _ = url.set_password(Some(&password));
        }
        */
        if self.inner.tls_config.is_some() {
          if self.inner.tls_config.as_ref().unwrap().accept_invalid_certificates {
            url.query_pairs_mut().append_pair("ssl-insecure", "yes");
          } else {
            url.query_pairs_mut().append_pair("ssl", "yes");
          }
        }
        url
    }
}
