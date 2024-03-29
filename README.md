<h1 align="center">RXQLite</h1>
<div align="center">
 <strong>
   A secure distributed SQLite Database
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
  
  [![CI](https://github.com/HaHa421/rxqlite/actions/workflows/ci.yaml/badge.svg?branch=byc)](https://github.com/HaHa421/rxqlite/actions/workflows/ci.yaml)
  
</div>

## Install

<br />

You can compile rxqlited or use binaries built for Windows or Ubuntu (x86_64) on github available at: https://github.com/HaHa421/rxqlite/releases.

In case of binaries, be sure to use to use the latest version.


Windows: you need to use the msvc toolchain (mingw build is broken due to https://github.com/rust-rocksdb/rust-rocksdb/issues/866)

<br />

## Usage

<br />
You will run rxqlited in cluster or in a single node mode, and since rxqlited uses the raft protocol, 
clusters must contain an odd number of nodes (rxqlited as a single node forms a cluster with only one node)

Starting a single node cluster on a local machine:
using
```bash
rxqlited --id 1 --http-addr 127.0.0.1:21001 --rpc-addr 127.0.0.1:22001 --notifications-addr 127.0.0.1:23001
```

rxqlited will listen on localhost:22001 for api and cluster management requests.

You can then get metrics from rxqlited using the following command:

```bash
curl http://localhost:21001/cluster/metrics
```

You should see the cluster current_leader as node 1, 
which is expected in a single node cluster.

Starting a 3 node cluster on a local machine:

```bash
rxqlited --id 1 --http-addr 127.0.0.1:21001 --rpc-addr 127.0.0.1:22001 --notifications-addr 127.0.0.1:23001 --member "2;127.0.0.1:21002;127.0.0.1:22002" --member "3;127.0.0.1:21003;127.0.0.1:22003" --leader true

rxqlited --id 2 --http-addr 127.0.0.1:21002 --rpc-addr 127.0.0.1:22002 --notifications-addr 127.0.0.1:23002

rxqlited --id 3 --http-addr 127.0.0.1:21003 --rpc-addr 127.0.0.1:22003 --notifications-addr 127.0.0.1:23003

```

will start three instances of rxqlited.

After a few seconds, we can check the cluster metrics using:

```bash
curl http://localhost:21001/cluster/metrics
```

and check that the cluster contais 3 nodes (membership : [1,2,3]).


Any subsequent cluster runs don't need to pass --member nor --leader when launching nodes

for further information on openraft you can check: https://github.com/datafuselabs/openraft

the client example shows a basic usage of the api using rust.

Furthermore, an sqlx driver is available from https://github.com/HaHa421/sqlx-rxqlite 
to ease the use of rxqlite using sqlx (https://github.com/launchbadge/sqlx)


## Security

WARNING: this database has not yet received any ITSA (Independent Third-party Security Audit)

This version of rxqlited supports tls in an insecure mode: 
  It accepts invalid certificates (this includes self-signed certificates)

```bash 
./ha-init-cluster-insecure.sh
```
shows how to run the cluster in insecure tls mode.

It will generate a self signed certificate and private key in certs-test and will use this pair
for all the nodes in the cluster (3 nodes here).

It then starts the three nodes with non tls parameters , adding
--cert-path for the certificate path,
--key-path for the private key path,
--accept-invalid-certificates true.
the parameter accept-invalid-certificates lets rxqlited accept invalid certificates.

Again, on subsequent cluster runs you dont need to pass all the initialisation parameters.
One needs only to provide node_id (provided that the data dir is 
the default ./data-{node-id} which is not an option in the current release) as shown in
ha-start-cluster.sh

## License

Licensed under either of

-   Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
-   MIT license
    ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any Contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.














