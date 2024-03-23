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

pub fn get_cluster_manager(test_name: &str,instance_count: usize)->anyhow::Result<TestClusterManager> {
  
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
  let working_directory=temp_dir.join(test_name);
  TestClusterManager::new(
    instance_count,
    &working_directory,
    &executable_path,
    "127.0.0.1",
    None,
  )
}



pub struct TestManager {
  pub tcm: TestClusterManager,
  pub clients: HashMap<NodeId,RXQLiteClient>,
}

impl std::ops::Deref for TestManager {
  type Target = TestClusterManager;
  fn deref(&self)->&Self::Target {
    &self.tcm
  }
}

impl std::ops::DerefMut for TestManager {
  fn deref_mut(&mut self)->&mut Self::Target {
    &mut self.tcm
  }
}

impl TestManager {
  pub fn new(test_name: &str,instance_count: usize)->Self {
    let tcm = get_cluster_manager(test_name,instance_count).unwrap();
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
  
  pub async fn wait_for_cluster_established(&self,
    node_id : NodeId,
    reattempts: usize,
  )->anyhow::Result<()> {
    let mut reattempts = reattempts+1; // wait max for cluster to establish
    
    loop {
      if let Ok(metrics) = self.get_metrics(node_id).await {
        let voter_ids=metrics.membership_config.voter_ids();
        if voter_ids.count() == self.node_count() {
          return Ok(());
        }
        
      }
      reattempts-=1;
      if reattempts == 0 {
        break;
      }
      std::thread::sleep(std::time::Duration::from_secs(1));
    }
    Err(anyhow::anyhow!("wait_for_cluster_established timeout"))
  }
  
}

#[test]
fn init_cluster() {
  
  let rt = Runtime::new().unwrap();
  
  let _= rt.block_on(async {
    let tm = TestManager::new("init_cluster",3);
    
    tm.wait_for_cluster_established(1,60).await.unwrap();
    
  });
}

#[test]
fn start_cluster() {
  
  let rt = Runtime::new().unwrap();
  
  let _= rt.block_on(async {
    let mut tm = TestManager::new("start_cluster",3);
    //let mut metrics: HashMap<NodeId,typ::RaftMetrics> = Default::default();
    
    tm.wait_for_cluster_established(1,60).await.unwrap();
    
    tm.kill_all().unwrap();
    
    tm.start().unwrap();
    
    tm.wait_for_cluster_established(1,60).await.unwrap();
    
  });
}
