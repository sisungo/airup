use cgroups_rs::{
    CgroupPid, cgroup_builder::CgroupBuilder, cpu::CpuController, memory::MemController,
};
use std::{
    io::ErrorKind,
    sync::{
        OnceLock,
        atomic::{self, AtomicU64},
    },
};

static CONTROLLER: OnceLock<RealmController> = OnceLock::new();

#[derive(Debug)]
struct RealmController {
    prefix: i64,
    id: AtomicU64,
}
impl RealmController {
    fn allocate_id(&self) -> u64 {
        self.id.fetch_add(1, atomic::Ordering::SeqCst)
    }
}

fn controller() -> &'static RealmController {
    CONTROLLER.get_or_init(|| RealmController {
        prefix: airupfx_time::timestamp_ms(),
        id: AtomicU64::new(1),
    })
}

#[derive(Debug)]
pub struct Realm {
    cg: cgroups_rs::Cgroup,
}
impl Realm {
    pub fn new() -> std::io::Result<Self> {
        Self::pid_detect()?;
        let ctrl = controller();
        let id = ctrl.allocate_id();
        let hier = cgroups_rs::hierarchies::auto();
        let cg = CgroupBuilder::new(&format!("airup_{}_{id}", ctrl.prefix))
            .cpu()
            .done()
            .memory()
            .done()
            .build(hier)
            .map_err(|x| std::io::Error::new(ErrorKind::PermissionDenied, x.to_string()))?;

        Ok(Self { cg })
    }

    pub fn set_cpu_limit(&self, max: u64) -> std::io::Result<()> {
        self.cg
            .controller_of::<CpuController>()
            .ok_or_else(|| std::io::Error::from(ErrorKind::PermissionDenied))?
            .set_shares(max)
            .map_err(|x| std::io::Error::new(ErrorKind::PermissionDenied, x.to_string()))?;

        Ok(())
    }

    pub fn set_mem_limit(&self, max: usize) -> std::io::Result<()> {
        self.cg
            .controller_of::<MemController>()
            .ok_or_else(|| std::io::Error::from(ErrorKind::PermissionDenied))?
            .set_limit(max as _)
            .map_err(|x| std::io::Error::new(ErrorKind::PermissionDenied, x.to_string()))?;

        Ok(())
    }

    pub fn add(&self, pid: i64) -> std::io::Result<()> {
        self.cg
            .add_task_by_tgid(CgroupPid::from(pid as u64))
            .map_err(|x| std::io::Error::new(ErrorKind::PermissionDenied, x.to_string()))?;

        Ok(())
    }

    pub fn kill(&self) -> std::io::Result<()> {
        self.cg
            .kill()
            .map_err(|x| std::io::Error::new(ErrorKind::PermissionDenied, x.to_string()))?;

        Ok(())
    }

    pub fn memory_usage(&self) -> std::io::Result<usize> {
        Ok(self
            .cg
            .controller_of::<MemController>()
            .ok_or_else(|| std::io::Error::from(ErrorKind::PermissionDenied))?
            .memory_stat()
            .usage_in_bytes as usize)
    }

    fn pid_detect() -> std::io::Result<()> {
        match std::process::id() {
            1 => Ok(()),
            _ => Err(ErrorKind::PermissionDenied.into()),
        }
    }
}
impl Drop for Realm {
    fn drop(&mut self) {
        _ = self.cg.delete();
    }
}
