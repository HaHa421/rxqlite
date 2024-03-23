use std::path::{PathBuf,Path};
use std::process::{Command, /*Stdio ,*/ Child};
use rcgen::generate_simple_self_signed;
use std::collections::HashMap;

use std::sync::{Arc, atomic::{AtomicU16, Ordering}};


#[cfg(target_os = "windows")]
const BASE_PORT:u16=21000;

#[cfg(target_os = "linux")]
const BASE_PORT:u16=22000;

pub struct PortManager {
  next_port : Arc<AtomicU16>,
}

impl Default for PortManager {
  fn default()->Self {
    Self {
      next_port: Arc::new(AtomicU16::new(BASE_PORT)),
    }
  }
}

impl PortManager {
  pub fn reserve(&self,port_count: usize)->u16 {
    self.next_port.fetch_add(port_count as _,Ordering::Relaxed)
  }
}

pub static PORT_MANAGER: state::InitCell<PortManager> = state::InitCell::new();


#[derive(Default,Clone)]
pub struct TestTlsConfig {
  pub accept_invalid_certificates: bool,
}

impl TestTlsConfig {
  pub fn accept_invalid_certificates(mut self,accept_invalid_certificates:bool)->Self {
    self.accept_invalid_certificates=accept_invalid_certificates;
    self
  }
}

pub struct Instance {
  pub node_id: u64,
  pub child: Option<Child>,
  pub data_path: PathBuf,
  pub http_addr: String,
}

pub struct TestClusterManager {
  pub instances: HashMap<u64,Instance>,
  pub tls_config: Option<TestTlsConfig>,
  pub working_directory: std::path::PathBuf,
  pub executable: String,
  pub keep_temp_directories: bool,
}

impl TestClusterManager {
  pub fn new<P: AsRef<Path>>(
    instance_count: usize,
    working_directory: P,
    executable_path: P,
    host: &str,
    tls_config: Option<TestTlsConfig>,
    )-> anyhow::Result<Self> {
    std::fs::create_dir_all(&working_directory)?;
    let base_port = PORT_MANAGER.get_or_init(Default::default).reserve(instance_count<<1);
  
    let (cert_path,key_path,accept_invalid_certificates) = if let Some(tls_config) = tls_config.as_ref() {
      let certs_path = working_directory.as_ref().join("certs-test");
      std::fs::create_dir_all(&certs_path)?;
      let subject_alt_names = vec![host.to_string()];

      let cert = generate_simple_self_signed(subject_alt_names)?;
      let key = cert.serialize_private_key_pem();
      let cert = cert.serialize_pem()?;
      let key_path = certs_path.join("rxqlited.key").to_path_buf();
      let cert_path = certs_path.join("rxqlited.cert").to_path_buf();
      
      std::fs::write(&key_path,key.as_bytes())?;
      std::fs::write(&cert_path,cert.as_bytes())?;
      (cert_path.to_str().unwrap().to_string(),key_path.to_str().unwrap().to_string(),tls_config.accept_invalid_certificates)
      
    } else {
      (Default::default(),Default::default(),false)
    };
    
    let mut instances: HashMap<u64,Instance> = Default::default();
    
    
    let executable= executable_path.as_ref();
    let executable= executable.to_str().unwrap().to_string();
    for i in 0..instance_count {
      
      
      let http_port= base_port + (i<<1) as u16;
      let rpc_port= base_port + ((i<<1) + 1) as u16;
      let http_addr=format!("{}:{}",host,http_port);
      let rpc_addr=format!("{}:{}",host,rpc_port);
      
      
      let mut cmd = Command::new(&executable);
      
        cmd
        //.stderr(Stdio::null())
        //.stdout(Stdio::null())
        //.env_clear()
        .arg("--id")
        .arg(&(i+1).to_string())
        .arg("--http-addr")
        .arg(&http_addr)
        .arg("--rpc-addr")
        .arg(&rpc_addr)
        .current_dir(&working_directory);
      
      if tls_config.is_some() {
        cmd.arg("--key-path")
          .arg(&key_path)
          .arg("--cert-path")
          .arg(&cert_path);
        if accept_invalid_certificates {
          cmd.arg("--accept-invalid-certificates");
        }
      }
      if i == 0 {
        cmd.arg("--leader");
        for j in 1..instance_count {
          cmd.arg("--member");
          let http_port= base_port + (j<<1) as u16;
          let rpc_port= base_port + ((j<<1) + 1) as u16;
          let http_addr=format!("{}:{}",host,http_port);
          let rpc_addr=format!("{}:{}",host,rpc_port);

          cmd.arg(format!("{};{};{}",j+1,http_addr,rpc_addr));
        }
      }
      let child = cmd.spawn()?;
      let node_id:u64 =  (i+1) as _;
      
      instances.insert(node_id,
        Instance{
          http_addr,
          node_id,
          child: Some(child),
          data_path: working_directory.as_ref().join(format!("data-{}",i+1)),
        }
      );
    }
    Ok(Self {
      instances,
      tls_config,
      working_directory:working_directory.as_ref().to_path_buf(),
      executable,
      keep_temp_directories: false,
    })
  }
  pub fn kill_all(&mut self)-> anyhow::Result<()> {
    for (_,instance) in self.instances.iter_mut() {
      if let Some(child) = instance.child.as_mut() {
        child.kill()?;
      }
    }
    loop {
      let mut done=true;
      for (_,instance) in self.instances.iter_mut() {
        if let Some(child) = instance.child.as_mut() {
          if let Ok(Some(_exit_status))=child.try_wait() {
            instance.child.take();
          } else {
            done = false;
          }
        }
      }
      if done {
        break;
      }
      std::thread::sleep(std::time::Duration::from_millis(250));
    }
    Ok(())
  }
  pub fn start(&mut self)-> anyhow::Result<()> {
    for (node_id,instance) in self.instances.iter_mut() {
      let mut cmd = Command::new(&self.executable);
      
      cmd.arg("--id")
        .arg(&node_id.to_string())
        .current_dir(&self.working_directory);
      let child = cmd.spawn()?;
      instance.child=Some(child);
    }
    Ok(())
  }
  pub fn clean_directories(&self)->anyhow::Result<()> {
    if self.keep_temp_directories {
      return Ok(());
    }
    if let Err(err) = std::fs::remove_dir_all(&self.working_directory) {
      eprintln!("error removing directory : {}({})",self.working_directory.display(),err);
      Err(err.into())
    } else {
      Ok(())
    }
  }
}

impl Drop for TestClusterManager {
  fn drop(&mut self) {
    let _= self.kill_all();
    let _= self.clean_directories();
  }
}