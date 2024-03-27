use super::*;

#[test]
fn init_cluster() {
    let rt = Runtime::new().unwrap();
    let _ = rt.block_on(async {
        let tm = TestManager::new("init_cluster", 3, None);

        tm.wait_for_cluster_established(1, 60).await.unwrap();
    });
}

#[test]
fn start_cluster() {
    let rt = Runtime::new().unwrap();

    let _ = rt.block_on(async {
        let mut tm = TestManager::new("start_cluster", 3, None);
        tm.wait_for_cluster_established(1, 60).await.unwrap();
        tm.kill_all().unwrap();
        tm.start().unwrap();
        tm.wait_for_cluster_established(1, 60).await.unwrap();
    });
}

#[test]
fn init_cluster_insecures_ssl() {
    let rt = Runtime::new().unwrap();
    let _ = rt.block_on(async {
        let tm = TestManager::new(
            "init_cluster_insecures_ssl",
            3,
            Some(TestTlsConfig::default().accept_invalid_certificates(true)),
        );

        tm.wait_for_cluster_established(1, 60).await.unwrap();
    });
}

#[test]
fn start_cluster_insecure_ssl() {
    let rt = Runtime::new().unwrap();

    let _ = rt.block_on(async {
        let mut tm = TestManager::new(
            "start_cluster_insecure_ssl",
            3,
            Some(TestTlsConfig::default().accept_invalid_certificates(true)),
        );
        //tm.keep_temp_directories=true;
        tm.wait_for_cluster_established(1, 60).await.unwrap();
        tm.kill_all().unwrap();
        tm.start().unwrap();
        tm.wait_for_cluster_established(1, 60).await.unwrap();
    });
}
