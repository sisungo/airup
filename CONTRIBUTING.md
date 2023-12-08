# Contributor's Manual
Welcome to contribute to Airup!

## HOWTO: Port Airup To A New Platform
Airup depends on some libraries which contains platform-specific code. All these dependencies must be satisfied:
 - `tokio`
 - `sysinfo`

Some OS features are used:
 - Unix Domain Sockets

All other platform-specific codes are wrapped by `airupfx::sys`.

## Licensing
By contributing, you agree to license your code under the same license as existing source code of `Airup`. View [the LICENSE file](LICENSE).