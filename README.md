<h1 align="center">RXQLite</h1>
<div align="center">
 <strong>
   A distributed SQLite Database
 </strong>
</div>
<br />


<div align="center">
  <h4>
    <a href="#install">
      Install
    </a>
    <span> | </span>
    <a href="#usage">
      Usage
    </a>
    <span> | </span>
    <a href="#security">
      Security
    </a>
    <span> | </span>
    <a href="#license">
      License
    </a>
  </h4>
</div>

<br />

<div align="center">
  
  <!-- Version -->
  <a href="https://crates.io/crates/rxqlite">
    <img src="https://img.shields.io/crates/v/rxqlite.svg?style=flat-square"
    alt="Crates.io version" /></a>
  
</div>

## Install

<br />

Right now , you need to compile rxsqlited

Windows: you need to use the msvc toolchain (mingw build is broken due to https://github.com/rust-rocksdb/rust-rocksdb/issues/866)

<br />

## Usage

<br />
You will run rxqlited in cluster or in a single node mode, and since rxqlited uses the raft protocol, 
clusters must contain an odd number of nodes (rxqlited as a single node forms a cluster with only one node)

Starting a single node cluster on a local machine:
using
```bash
rxqlited --id 1 --http-addr 127.0.0.1:21001 --rpc-addr 127.0.0.1:22001
```

rxqlited will listen on localhost:22001 for api and cluster management requests.

On first cluster run, the leader needs to be initiated as follow:

```bash
curl http://localhost:21001/cluster/init -H "Content-Type: application/json" -d '{}'
```

then you can get metrics from rxqlited using the following command:

```bash
curl http://localhost:21001/cluster/metrics
```

you should see the cluster current_leader as node 1, which is expected in a single node cluster.

Starting a 3 node cluster on a local machine:

```bash
rxqlited --id 1 --http-addr 127.0.0.1:21001 --rpc-addr 127.0.0.1:22001

rxqlited --id 2 --http-addr 127.0.0.1:21002 --rpc-addr 127.0.0.1:22002

rxqlited --id 3 --http-addr 127.0.0.1:21003 --rpc-addr 127.0.0.1:22003

```

will start three instances of rxqlited.

On first cluster run , we need to initialize the cluster:
we first init the leader as above:
```bash
curl http://localhost:21001/cluster/init -H "Content-Type: application/json" -d '{}'
```
and then add node 2 and node 3 as learners using the cluster api of node 1 (that is on port 21001):


```bash
curl http://localhost:21001/cluster/add-learner -H "Content-Type: application/json" -d '[2, "127.0.0.1:21002", "127.0.0.1:22002"]'

curl http://localhost:21001/cluster/add-learner -H "Content-Type: application/json" -d '[3, "127.0.0.1:21003", "127.0.0.1:22003"]'

```

We then change the cluster membership from one node (node 1) to three nodes using:

```bash
curl http://localhost:21001/cluster/change-membership -H "Content-Type: application/json" -d '[1, 2, 3]'

```
After a few seconds, we can check the cluster metrics using:

```bash
curl http://localhost:21001/cluster/metrics
```

and check that the cluster contais 3 nodes (membership : [1,2,3]).


we are now done with cluster first run initialisation.

any subsequent cluster runs don't need to go through cluster initialisation step.

for further information on openraft you can check: https://github.com/datafuselabs/openraft

the client example shows a basic usage of the api using rust.

Furthermore, an sqlx driver is available from https://github.com/HaHa421/rxqlite/tree/main/crates/sqlx-rxqlite 
to ease the use of rxqlite using sqlx (https://github.com/launchbadge/sqlx)


## Security

This version of rxqlited doesn't support tls yet
 

## License

Licensed under

-   Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
.

## Contribution

Unless you explicitly state otherwise, any Contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.

