#[derive(Debug)]
pub struct Realm;
impl Realm {
    pub fn new() -> std::io::Result<Self> {
        Ok(Self)
    }

    pub fn set_cpu_limit(&self, _: u64) -> std::io::Result<()> {
        Ok(())
    }

    pub fn set_mem_limit(&self, _: usize) -> std::io::Result<()> {
        Ok(())
    }

    pub fn add(&self, _: i64) -> std::io::Result<()> {
        Ok(())
    }

    pub fn kill(&self) -> std::io::Result<()> {
        Ok(())
    }

    pub fn memory_usage(&self) -> std::io::Result<usize> {
        Ok(0)
    }
}
