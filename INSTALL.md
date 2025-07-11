# Airup Installation Guide
This guide describes how to build, configure and install Airup.

## Platform Support
Airup compiles on all Unix-like operating systems. As a service supervisor, it has been tested to work on Linux, macOS,
Android and FreeBSD. As an init system \(`pid == 1`\), it has been tested to work on Linux.

## Build Dependencies
To build Airup, you need to install the few dependencies first:
 - [Rust](https://rust-lang.org): The programming language used to implement Airup.

Airup requires `Rust 1.85.0` or newer to compile.

## Configuration
Some Airup functions are configured at build time. The build manifest which is located at `build_manifest.json` stores primitive
paths that cannot be set at runtime or default values of `system.conf` items. An example file is located
at `docs/resources/build_manifest.json`. There is a list that describes all available configuration items:
 - `os_name`: Name of the OS build.
 - `config_dir`: Path of Airup's configuration directory, which stores service/system config files, etc.
 - `service_dir`: Path of Airup's service directory, which stores service files.
 - `milestone_dir`: Path of Airup's milestone directory, which stores milestone files.
 - `runtime_dir`: Path of Airup's runtime directory, which stores runtime files like the Unix sockets, locks, etc.
 - `env_vars`: Environment variables for the global system. When a value is explictly set to `null`, the variables is deleted if
 it exists.
 - `early_cmds`: Commands that are executed in the `early_boot` pseudo-milestone.

## Build
To build debug version of Airup, run:
```shell
cargo build
```

To build release version of Airup, run:
```shell
cargo build --release
```

## Install
A standard Airup installation consists of the following files:
 - `airupd`: The main Airup daemon binary.
 - `airup`: A CLI utility to inspect or manipulate Airup components.
 - \[`airup-fallback-logger`\]: An Airup extension that implements a simple logger for the Airup Logger Interface for fallback use.
 This is not subject to be executed directly by the user and is usually placed at `/usr/libexec/airup/fallback-logger`.
 - `libairup_sdk.so` OR `libairup_sdk.dylib`: The Airup SDK for C, in dynamic library.
 - \[`docs/resources/airup-fallback-logger.airs`\]: Service manifest file for the `fallback-logger` service.
 - \[`docs/resources/airupd.airs`\]: Stub service manifest file for the `airupd` service.
 - \[`docs/resources/selinux/airup.te`\]: SELinux policy for Airup.

Read the [documents](docs/README.md) to learn more about installation.
