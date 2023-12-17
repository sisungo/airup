use super::*;
use crate::supervisor::SupervisorContext;
use airup_sdk::Error;
use airupfx::prelude::*;
use std::sync::Arc;

#[derive(Debug)]
pub struct HealthCheckHandle {
    helper: TaskHelperHandle,
}
impl HealthCheckHandle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_context: Arc<SupervisorContext>) -> Arc<dyn TaskHandle> {
        let (handle, helper) = task_helper();

        let reload_service = HealthCheck { helper, _context };
        reload_service.start();

        Arc::new(Self { helper: handle })
    }
}
impl TaskHandle for HealthCheckHandle {
    fn task_class(&self) -> &'static str {
        "HealthCheck"
    }

    fn is_important(&self) -> bool {
        false
    }

    fn send_interrupt(&self) {
        self.helper.send_interrupt()
    }

    fn wait(&self) -> BoxFuture<Result<TaskFeedback, Error>> {
        self.helper.wait()
    }
}

#[derive(Debug)]
struct HealthCheck {
    helper: TaskHelper,
    _context: Arc<SupervisorContext>,
}
impl HealthCheck {
    fn start(mut self) {
        tokio::spawn(async move {
            let val = self.run().await;
            self.helper.finish(val);
        });
    }

    async fn run(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
