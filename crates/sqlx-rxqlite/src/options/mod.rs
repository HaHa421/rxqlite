use rxqlite::RSQliteClientTlsConfig;

#[derive(Debug, Clone)]
pub struct RXQLiteConnectOptions {
    pub(crate) inner: rxqlite::ConnectOptions,
}

impl RXQLiteConnectOptions {
  pub fn leader_id(mut self, leader_id: u64) -> Self {
    self.inner.leader_id = leader_id;
    self
  }
  pub fn host(mut self, host: &str) -> Self {
    self.inner.leader_host = host.to_owned();
    self
  }
  pub fn port(mut self, port: u16) -> Self {  
    self.inner.leader_port = port;
    self
  }
  
  /*
  pub fn user(mut self, user: Option<String>) -> Self {  
    self.inner.user = user;
    self
  }
  pub fn password(mut self, pwd: Option<String>) -> Self {  
    self.inner.pass = pwd;
    self
  }
  */
  pub fn use_ssl(mut self, use_ssl: bool) -> Self {  
    if use_ssl {
      //rxqlite::Scheme::HTTPS
      self.inner.tls_config = Some(Default::default());
      self.inner.tls_config.as_mut().unwrap().accept_invalid_certificates = false;
    } else {
      self.inner.tls_config = None;
    }
    self
  }
  pub fn use_insecure_ssl(mut self, use_insecure_ssl: bool) -> Self {
    if self.inner.tls_config.is_none() {
      self.inner.tls_config = Some(Default::default());
    }
    if use_insecure_ssl {
      //self.inner.scheme = rxqlite::Scheme::HTTPS;
      self.inner.tls_config.as_mut().unwrap().accept_invalid_certificates = true;
    } else {
      self.inner.tls_config.as_mut().unwrap().accept_invalid_certificates = false;
      //self.inner.scheme = rxqlite::Scheme::HTTP;
    }
    self
  }
  pub fn add_cert_path(mut self, cert_path: String) -> Self {
    if self.inner.tls_config.is_none() {
      self.inner.tls_config = Some(Default::default());
    }
    self.inner.tls_config.as_mut().unwrap().cert_paths.push(cert_path);
    self
  }
  pub fn tls_config(mut self, tls_config: Option<RSQliteClientTlsConfig>) -> Self {
    self.inner.tls_config =  tls_config;
    self
  }
}

pub mod connect;
mod parse;
