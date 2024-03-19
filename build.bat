rustup default  stable-x86_64-pc-windows-msvc

set PATH=%PATH%;C:\Program Files\LLVM\bin

cargo build --release
cargo build --release --example client

rustup default  stable-x86_64-pc-windows-gnu

