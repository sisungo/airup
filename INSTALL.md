# Airup Installation Guide
This guide describes how to build, configure and install Airup.

## Before Building
To build Airup, you must install `Rust` first. It can be simply installed with:
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Some build dependencies are optional, but is useful for some workflows:
 - [cargo-make](https://github.com/sagiegurari/cargo-make): The workflow manager.
 - [cbindgen](https://github.com/mozilla/cbindgen): Generates C headers.

## Configuration
Some Airup functions are configured at build time. The build manifest which is located at `build_manifest.json` stores primitive
configuration items that cannot be set at runtime. Its example is at `build_manifest.json.example`. Definitions to its items:
 - `config_dir`: Path of Airup's configuration directory, which stores security policy, services, milestones, system configuration, etc.
 - `runtime_dir`: Path of Airup's runtime directory, which stores runtime files like the Unix socket.
 - `default_system_conf`: Default content of `system.conf`, represented in JSON.

## Build
Build debug version of Airup with command:
```shell
cargo build
```

To build release version of Airup, run:
```shell
cargo build --release
```