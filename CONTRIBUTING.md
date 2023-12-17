# Contributor's Manual
Welcome to contribute to Airup!

## Developer Guide
For debugging Airup, this `build_manifest.json` would help a lot:

```json
{
    "os_name": "\u001b[36;4mAirup\u001b[0m",
    "config_dir": "target/airup_data/config",
    "service_dir": "target/airup_data/services",
    "milestone_dir": "target/airup_data/milestones",
    "runtime_dir": "/tmp/airup_dev",
    "log_dir": "target/airup_data/logs",
    "env_vars": {},
    "early_cmds": []
}
```

## HOWTO: Port Airup To A New Platform
Airup depends on some libraries which contains platform-specific code. All these dependencies must be satisfied:
 - `tokio`
 - `sysinfo`

Some OS features are used:
 - Unix Domain Sockets

All other platform-specific codes are wrapped by `airupfx::sys`.

## Licensing
By contributing, you agree to license your code under the same license as existing source code of `Airup`. View [the LICENSE file](LICENSE).