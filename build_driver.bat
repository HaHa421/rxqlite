rustup default  stable-x86_64-pc-windows-msvc

set PATH=%PATH%;C:\Program Files\LLVM\bin

cargo build --release -p sqlx-rxqlite --features=chrono --example simple --example simple_chrono

rustup default  stable-x86_64-pc-windows-gnu

