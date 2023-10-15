# Airup Installation Guide
This guide describes how to build, configure and install Airup.

## Before Building
To build Airup, you must install `Rust` first. It can be simply installed with:
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Some build dependencies are optional, but is useful for some workflows:
 - [cargo-make](https://github.com/sagiegurari/cargo-make): A workflow manager for Rust.

## Configuration
Some Airup functions are configured at build time. The build manifest which is located at `build_manifest.rs` stores primitive
configuration items that cannot be set at runtime. Its example is at `docs/resources/build_manifest.rs`. Definitions to its
items:
 - `os_name`: Name of the OS build.
 - `config_dir`: Path of Airup's configuration directory, which stores security policy, system configuration, etc.
 - `service_dir`: Path of Airup's service directory, which stores services.
 - `milestone_dir`: Path of Airup's milestone directory, which stores milestones.
 - `runtime_dir`: Path of Airup's runtime directory, which stores runtime files like the Unix socket.
 - `env_vars`: Environment variables for the global system. When a value is explictly set to `null`, the variables is deleted if it exists.
 - `early_cmds`: Commands that are executed in the `early_boot` pseudo-milestone.
 - `security`: Determines which security model is used by default.

## Build
Build debug version of Airup with command:
```shell
cargo build
```

To build release version of Airup, run:
```shell
cargo build --release
```