//! APIs that provides system operations.

use super::{
    util::{ok, ok_null},
    Method, MethodFuture,
};
use crate::{app::airupd, ipc::SessionContext};
use airup_sdk::ipc::Request;
use std::{collections::HashMap, hash::BuildHasher, sync::Arc};

pub fn init<H: BuildHasher>(methods: &mut HashMap<&'static str, Method, H>) {
    methods.insert("system.refresh", refresh);
    methods.insert("system.gc", gc);
    methods.insert("system.start_service", start_service);
    methods.insert("system.query_service", query_service);
    methods.insert("system.query_system", query_system);
    methods.insert("system.stop_service", stop_service);
    methods.insert("system.kill_service", kill_service);
    methods.insert("system.reload_service", reload_service);
    methods.insert("system.sideload_service", sideload_service);
    methods.insert("system.unsideload_service", unsideload_service);
    methods.insert("system.cache_service", cache_service);
    methods.insert("system.uncache_service", uncache_service);
    methods.insert("system.interrupt_service_task", interrupt_service_task);
    methods.insert("system.list_services", list_services);
    methods.insert("system.use_logger", use_logger);
    methods.insert("system.tail_logs", tail_logs);
    methods.insert("system.enter_milestone", enter_milestone);
    methods.insert("system.poweroff", poweroff);
    methods.insert("system.reboot", reboot);
    methods.insert("system.halt", halt);
}

fn refresh(_: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move {
        airupfx::env::refresh().await;
        airupd().supervisors.refresh_all().await;

        ok_null()
    })
}

fn gc(_: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move {
        airupd().supervisors.gc().await;
        ok_null()
    })
}

fn query_service(_: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let service: String = req.extract_params()?;
        ok(airupd().query_service(&service).await?)
    })
}

fn query_system(_: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move { ok(airupd().query_system().await) })
}

fn start_service(_: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let service: String = req.extract_params()?;
        let handle = airupd().start_service(&service).await?;
        handle.wait().await?;
        ok_null()
    })
}

fn stop_service(_: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let service: String = req.extract_params()?;
        airupd().stop_service(&service).await?.wait().await?;
        ok_null()
    })
}

fn kill_service(_: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let service: String = req.extract_params()?;
        airupd().stop_service(&service).await?.wait().await?;
        ok_null()
    })
}

fn reload_service(_: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let service: String = req.extract_params()?;
        airupd().reload_service(&service).await?.wait().await?;
        ok_null()
    })
}

fn interrupt_service_task(_: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let service: String = req.extract_params()?;
        airupd()
            .interrupt_service_task(&service)
            .await?
            .wait()
            .await?;
        ok_null()
    })
}

fn sideload_service(_: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let (name, service): (String, _) = req.extract_params()?;
        airupd().storage.services.load(&name, service)?;
        ok_null()
    })
}

fn unsideload_service(_: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let name: String = req.extract_params()?;
        airupd().storage.services.unload(&name)?;
        ok_null()
    })
}

fn cache_service(_: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let service: String = req.extract_params()?;
        airupd().cache_service(&service).await?;
        ok_null()
    })
}

fn uncache_service(_: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let service: String = req.extract_params()?;
        airupd().uncache_service(&service).await?;
        ok_null()
    })
}

fn list_services(_: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move { ok(airupd().storage.services.list().await) })
}

fn use_logger(_: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let logger: Option<String> = req.extract_params()?;
        if let Some(name) = logger {
            airupd().logger.set_logger_by_name(&name).await?;
        } else {
            airupd().logger.remove_logger().await;
        }
        ok_null()
    })
}

fn tail_logs(_: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let (subject, n): (String, usize) = req.extract_params()?;
        let queried = airupd()
            .logger
            .tail(&subject, n)
            .await
            .map_err(airup_sdk::Error::custom)?;
        ok(queried)
    })
}

fn enter_milestone(_: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let name: String = req.extract_params()?;
        airupd().enter_milestone(name).await?;
        ok_null()
    })
}

fn poweroff(_: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move {
        airupd().lifetime.poweroff();
        ok_null()
    })
}

fn reboot(_: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move {
        airupd().lifetime.reboot();
        ok_null()
    })
}

fn halt(_: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move {
        airupd().lifetime.halt();
        ok_null()
    })
}
