

#[derive(Debug, Clone)]
pub struct RaftSqliteConnectOptions {
    pub(crate) inner: rxqlite::ConnectOptions,
}

impl RaftSqliteConnectOptions {
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
    self.inner.scheme = if use_ssl {
      rxqlite::Scheme::HTTPS
    } else {
      rxqlite::Scheme::HTTP
    };
    self.inner.accept_invalid_cert=false;
    self
  }
  pub fn use_insecure_ssl(mut self, use_insecure_ssl: bool) -> Self {
    if use_insecure_ssl {
      self.inner.scheme = rxqlite::Scheme::HTTPS;
      self.inner.accept_invalid_cert = true;
    } else {
      self.inner.accept_invalid_cert = false;
      self.inner.scheme = rxqlite::Scheme::HTTP;
    }
    self
  }
}

pub mod connect;
mod parse;
