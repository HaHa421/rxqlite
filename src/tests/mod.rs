use std::path::PathBuf;
use std::env;
use std::collections::HashMap;
use rxqlite_tests_common::*;
use crate::typ;
use crate::NodeId;
use crate::client::RXQLiteClient;
use tokio::runtime::Runtime;

#[cfg(target_os = "windows")]
const EXE_SUFFIX:&str=".exe";

#[cfg(not(target_os = "windows"))]
const EXE_SUFFIX:&str="";

#[cfg(target_os = "windows")]
const BASE_PORT:u16=21000;

#[cfg(target_os = "linux")]
const BASE_PORT:u16=22000;


pub fn default_cluster_manager()->anyhow::Result<TestClusterManager> {
  
  let executable_path = if let Ok(rxqlited_dir) = std::env::var("RXQLITED_DIR") {
    
    let executable_path = PathBuf::from(rxqlited_dir).join(format!("rxqlited{}",EXE_SUFFIX));
    println!("using rxqlited: {}",executable_path.display());
    executable_path
  } else {
    let cargo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let executable_path = cargo_path.join("target").join("release").join(format!("rxqlited{}",EXE_SUFFIX));
    println!("using rxqlited: {}",executable_path.display());
    executable_path
  };
  assert!(executable_path.is_file());
  
  let temp_dir = env::temp_dir();
  let working_directory=temp_dir.join("init_cluster");
  TestClusterManager::new(
    3,
    &working_directory,
    &executable_path,
    "127.0.0.1",
    BASE_PORT,
    None,
  )
}



pub struct TestManager {
  pub tcm: TestClusterManager,
  pub clients: HashMap<NodeId,RXQLiteClient>,
}



impl TestManager {
  pub fn new(tcm: TestClusterManager)->Self {
    let clients: HashMap<NodeId,RXQLiteClient> = tcm.instances.iter().map(|(node_id,instance)|
      (
        *node_id,
        RXQLiteClient::new(instance.node_id, instance.http_addr.clone())
      )
    ).collect();
    Self {
      tcm,
      clients,
    }
  }
  
  pub async fn get_metrics(&self,node_id: NodeId)->anyhow::Result<typ::RaftMetrics> {
    let client = self.clients.get(&node_id).unwrap();
    let metrics=client.metrics().await?;
    Ok(metrics)
  }
  pub fn node_count(&self)->usize {
    self.tcm.instances.len()
  }
}

#[test]
fn init_cluster() {
  
  let rt = Runtime::new().unwrap();
  
  let _= rt.block_on(async {
    let tm = TestManager::new(
      default_cluster_manager().unwrap()
    );
    //let mut metrics: HashMap<NodeId,typ::RaftMetrics> = Default::default();
    
    let mut max_wait_loop = 2 * 60; // wait max for cluster to establish
    
    loop {
      if let Ok(metrics) = tm.get_metrics(1).await {
        let voter_ids=metrics.membership_config.voter_ids();
        if voter_ids.count() == tm.node_count() {
          break;
        }
        
      }
      max_wait_loop-=1;
      assert_ne!(max_wait_loop,0);
      std::thread::sleep(std::time::Duration::from_secs(1));
    }
    
    //assert_eq!(metrics.len(),tcm.instances.len());
    
    
  });
}