#[derive(Debug)]
pub struct Realm;
impl Realm {
    pub async fn new() -> std::io::Result<Self> {
        Ok(Self)
    }

    pub async fn set_cpu_limit(&self, _: u64) -> std::io::Result<()> {
        Ok(())
    }

    pub async fn set_mem_limit(&self, _: usize) -> std::io::Result<()> {
        Ok(())
    }

    pub async fn add(&self, _: i64) -> std::io::Result<()> {
        Ok(())
    }

    pub async fn kill(&self) -> std::io::Result<()> {
        Ok(())
    }
}
