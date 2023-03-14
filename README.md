# Windows Service in Rust

## Getting started

Run:

```
cargo build --release
```

on windows, if you are cross-compiling from linux, just indicate the relevant --target parameter.

You will have three exe's:

```
/target/release/install_service
/target/release/uninstall_service
/target/release/ping_service
```

Run `install_service` from `cmd` to install the service, once you've done that, you can start the service using `net start ping_service`

To uninstall use the `uninstall_service` exe.

## What it does

Initially, the program opens up a mpsc channel, which is used for the primary motive of stopping the service if necessary. Channel is created, a handler is setup to handle the 
Windows shutdown command and then two threads are spawned on a tokio runtime, which blocks the main thread until one of the threads completes. The only thread that can 
complete is the thread that pings the mpsc channel, awaiting a shutdown command. The other threads sends a GET request to a specified URL(in the constants at the top) every 10 
seconds. 

