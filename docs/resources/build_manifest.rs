build_manifest! {
    os_name: "\x1b[36;4mAirup\x1b[0m",
    config_dir: "/etc/airup",
    service_dir: "/etc/airup/services",
    milestone_dir: "/etc/airup/milestones",
    runtime_dir: "/run/airup",
    env_vars: {},
    early_cmds: [],
    security: Policy,
}