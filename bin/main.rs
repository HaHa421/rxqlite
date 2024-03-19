#![deny(warnings)]

use clap::Parser;
use rxqlite::start_example_raft_node;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Clone, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Opt {
    #[clap(long)]
    pub id: u64,

    #[clap(long)]
    pub http_addr: String,

    #[clap(long)]
    pub rpc_addr: String,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Setup the logger
    tracing_subscriber::fmt()
        //.with_max_level(tracing::Level::TRACE)
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true)
        .with_ansi(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Parse the parameters passed by arguments.
    let options = Opt::parse();
    let base_path = format!("data-{}", options.id);
    start_example_raft_node(
        options.id,
        //format!("data-{}/{}.db", options.id,options.rpc_addr),
        base_path,
        options.http_addr,
        options.rpc_addr,
    )
    .await
}
