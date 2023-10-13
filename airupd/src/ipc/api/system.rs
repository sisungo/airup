//! APIs that provides system operations.

use super::{
    util::{check_perm, ok, ok_null},
    Method, MethodFuture,
};
use crate::{app::airupd, ipc::SessionContext};
use airup_sdk::ipc::Request;
use airupfx::policy::Action;
use std::{collections::HashMap, hash::BuildHasher, sync::Arc};

pub fn init<H: BuildHasher>(methods: &mut HashMap<&'static str, Method, H>) {
    methods.insert("system.refresh", refresh);
    methods.insert("system.gc", gc);
    methods.insert("system.start_service", start_service);
    methods.insert("system.query_service", query_service);
    methods.insert("system.query_system", query_system);
    methods.insert("system.stop_service", stop_service);
    methods.insert("system.reload_service", reload_service);
    methods.insert("system.sideload_service", sideload_service);
    methods.insert("system.unsideload_service", unsideload_service);
    methods.insert("system.cache_service", cache_service);
    methods.insert("system.uncache_service", uncache_service);
    methods.insert("system.interrupt_service_task", interrupt_service_task);
    methods.insert("system.list_services", list_services);
    methods.insert("system.poweroff", poweroff);
    methods.insert("system.reboot", reboot);
    methods.insert("system.halt", halt);
}

fn refresh(context: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move {
        check_perm(&context, &[Action::Refresh]).await?;
        airupd().storage.config.policy.refresh().await;
        airupfx::env::refresh().await;
        ok_null()
    })
}

fn gc(context: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move {
        check_perm(&context, &[Action::Refresh]).await?;
        airupd().supervisors.gc().await;
        ok_null()
    })
}

fn query_service(context: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let service: String = req.extract_params()?;
        check_perm(&context, &[Action::QueryServices]).await?;
        ok(airupd().query_service(&service).await?)
    })
}

fn query_system(context: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move {
        check_perm(&context, &[Action::QueryServices]).await?;
        ok(airupd().query_system().await)
    })
}

fn start_service(context: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let service: String = req.extract_params()?;
        check_perm(&context, &[Action::ManageServices]).await?;
        let handle = airupd().start_service(&service).await?;
        handle.wait().await?;
        ok_null()
    })
}

fn stop_service(context: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let service: String = req.extract_params()?;
        check_perm(&context, &[Action::ManageServices]).await?;
        airupd().stop_service(&service).await?.wait().await?;
        ok_null()
    })
}

fn reload_service(context: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let service: String = req.extract_params()?;
        check_perm(&context, &[Action::ManageServices]).await?;
        airupd().reload_service(&service).await?.wait().await?;
        ok_null()
    })
}

fn interrupt_service_task(context: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let service: String = req.extract_params()?;
        check_perm(&context, &[Action::ManageServices]).await?;
        airupd()
            .interrupt_service_task(&service)
            .await?
            .wait()
            .await?;
        ok_null()
    })
}

fn sideload_service(context: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let (name, service): (String, _) = req.extract_params()?;
        check_perm(&context, &[Action::SideloadServices]).await?;
        airupd().storage.services.load(&name, service)?;
        ok_null()
    })
}

fn unsideload_service(context: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let name: String = req.extract_params()?;
        check_perm(&context, &[Action::SideloadServices]).await?;
        airupd().storage.services.unload(&name)?;
        ok_null()
    })
}

fn cache_service(context: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let service: String = req.extract_params()?;
        check_perm(&context, &[Action::ManageServices]).await?;
        airupd().cache_service(&service).await?;
        ok_null()
    })
}

fn uncache_service(context: Arc<SessionContext>, req: Request) -> MethodFuture {
    Box::pin(async move {
        let service: String = req.extract_params()?;
        check_perm(&context, &[Action::ManageServices]).await?;
        airupd().uncache_service(&service).await?;
        ok_null()
    })
}

fn list_services(context: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move {
        check_perm(&context, &[Action::QueryServices]).await?;
        ok(airupd().storage.services.list().await)
    })
}

fn poweroff(context: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move {
        check_perm(&context, &[Action::Power]).await?;
        airupd().lifetime.poweroff();
        ok_null()
    })
}

fn reboot(context: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move {
        check_perm(&context, &[Action::Power]).await?;
        airupd().lifetime.reboot();
        ok_null()
    })
}

fn halt(context: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move {
        check_perm(&context, &[Action::Power]).await?;
        airupd().lifetime.halt();
        ok_null()
    })
}
