//! # The Airup Supervisor
//! Main module containing full airup supervisor logic.

pub mod task;

use crate::app::airupd;
use ahash::AHashMap;
use airup_sdk::{
    files::{service::WatchdogKind, Service},
    system::{QueryService, Status},
    Error,
};
use airupfx::{ace::Child, io::PiperHandle, process::Wait};
use std::{
    cmp,
    sync::{
        atomic::{self, AtomicBool, AtomicI32, AtomicI64},
        Arc, Mutex, RwLock,
    },
    time::Duration,
};
use task::*;
use tokio::{
    sync::{mpsc, oneshot},
    task::AbortHandle,
};

macro_rules! supervisor_req {
    ($name:ident, $ret:ty, $req:expr) => {
        pub async fn $name(&self) -> $ret {
            let (tx, rx) = oneshot::channel();
            self.sender.send($req(tx)).await.unwrap();
            rx.await.unwrap()
        }
    };
}
macro_rules! start_task {
    ($name:ident, $type:ty) => {
        fn $name(&mut self, context: Arc<SupervisorContext>) -> Result<Arc<dyn TaskHandle>, Error> {
            self._start_task(context, <$type>::new)
        }
    };
}

/// A manager of Airup supervisors.
#[derive(Debug, Default)]
pub struct Manager {
    supervisors: tokio::sync::RwLock<AHashMap<String, Arc<SupervisorHandle>>>,
    provided: tokio::sync::RwLock<AHashMap<String, Arc<SupervisorHandle>>>,
}
impl Manager {
    /// Creates a new, empty [`Manager`] instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Supervises the given service in the supervisor set.
    pub async fn supervise(&self, service: Service) -> Arc<SupervisorHandle> {
        let mut lock = self.supervisors.write().await;

        if let Some(x) = lock.get(&service.name).cloned() {
            return x;
        }

        let name = service.name.clone();

        let provided = service.service.provides.clone();
        let handle = SupervisorHandle::new(service);
        lock.insert(name, handle.clone());

        let mut lock = self.provided.write().await;
        for i in provided {
            lock.insert(i, handle.clone());
        }

        handle
    }

    /// Gets a supervisor in the set.
    pub async fn get(&self, name: &str) -> Option<Arc<SupervisorHandle>> {
        let name = name.strip_suffix(".airs").unwrap_or(name);

        if let Some(name) = name.strip_suffix(".provided") {
            return self.provided.read().await.get(name).cloned();
        }

        self.supervisors.read().await.get(name).cloned()
    }

    /// Gets a list of names of supervisors.
    pub async fn list(&self) -> Vec<String> {
        self.supervisors
            .read()
            .await
            .keys()
            .map(|x| x.into())
            .collect()
    }

    /// Removes the specified supervisor.
    pub async fn remove(&self, name: &str) -> Result<(), Error> {
        let mut supervisors = self.supervisors.write().await;
        let mut provided = self.provided.write().await;
        Self::_remove_from(name, &mut supervisors, &mut provided, true).await
    }

    /// Removes supervisors that are not used.
    pub async fn gc(&self) {
        let mut supervisors = self.supervisors.write().await;
        let mut provided = self.provided.write().await;
        let all: Vec<_> = supervisors.keys().map(ToString::to_string).collect();

        // [`Manager::_remove_from`] fails if the specific supervisor cannot be removed, so we can iterate over all supervisors
        // to "try to remove".
        for k in all {
            Self::_remove_from(&k, &mut supervisors, &mut provided, false)
                .await
                .ok();
        }
    }

    /// Removes the specific supervisor from passed supervisor and provider set.
    ///
    /// It can sucessfully remove a service which is neither active, having errors or being registered as a provider. In
    /// permissive mode, it allows removing any non-active services, and automatically unregisters providers registered by
    /// the specific service.
    async fn _remove_from(
        name: &str,
        supervisors: &mut AHashMap<String, Arc<SupervisorHandle>>,
        provided: &mut AHashMap<String, Arc<SupervisorHandle>>,
        permissive: bool,
    ) -> Result<(), Error> {
        let handle = supervisors.get(name).ok_or(Error::UnitNotStarted)?.clone();
        let queried = handle.query().await;

        let is_providing = |provided: &mut AHashMap<_, _>, i| {
            if let Some(provided_handle) = provided.get(i) {
                if Arc::ptr_eq(&handle, provided_handle) {
                    return true;
                }
            }
            false
        };
        let is_provider = queried
            .definition
            .service
            .provides
            .iter()
            .map(|x| is_providing(provided, x))
            .any(|x| x);

        let removable = queried.status == Status::Stopped
            && queried.task_class.is_none()
            && (permissive || (queried.last_error.is_none() && !is_provider));

        if removable {
            supervisors.remove(name).unwrap();
            for i in &queried.definition.service.provides {
                if is_providing(provided, i) {
                    provided.remove(i);
                }
            }
            Ok(())
        } else if queried.status != Status::Stopped {
            Err(Error::UnitStarted)
        } else if queried.task_class.is_some() {
            Err(Error::TaskExists)
        } else {
            Err(Error::Internal {
                message: "something is provided by this or `last_error` is set".into(),
            })
        }
    }

    /// Refreshes all running supervisors. Returns a list of services that was not successfully refreshed.
    pub async fn refresh_all(&self) -> Vec<String> {
        let supervisors = self.supervisors.read().await;
        let mut errors = vec![];

        for (k, v) in &*supervisors {
            let queried = v.query().await;
            if !queried.definition.paths.is_empty() {
                let new = Service::read_merge(&queried.definition.paths).await;
                let new = match new {
                    Ok(x) => x,
                    Err(_) => {
                        errors.push(k.into());
                        continue;
                    }
                };
                if v.update_def(new).await.is_err() {
                    errors.push(k.into());
                }
            }
        }

        errors
    }
}

/// Handle of an Airup supervisor.
#[derive(Debug)]
pub struct SupervisorHandle {
    sender: mpsc::Sender<Request>,
}
impl SupervisorHandle {
    /// Creates a new [`SupervisorHandle`] instance while starting the associated supervisor.
    pub fn new(service: Service) -> Arc<Self> {
        let (sender, receiver) = mpsc::channel(128);

        let supervisor = Supervisor {
            receiver,
            current_task: CurrentTask::default(),
            context: SupervisorContext::new(service),
            timers: Box::default(),
        };
        supervisor.start();

        Arc::new(Self { sender })
    }

    supervisor_req!(query, QueryService, Request::Query);
    supervisor_req!(start, Result<Arc<dyn TaskHandle>, Error>, Request::Start);
    supervisor_req!(stop, Result<Arc<dyn TaskHandle>, Error>, Request::Stop);
    supervisor_req!(reload, Result<Arc<dyn TaskHandle>, Error>, Request::Reload);
    supervisor_req!(
        get_task,
        Result<Arc<dyn TaskHandle>, Error>,
        Request::GetTaskHandle
    );
    supervisor_req!(
        interrupt_task,
        Result<Arc<dyn TaskHandle>, Error>,
        Request::InterruptTask
    );
    supervisor_req!(
        make_active_raw,
        Result<Arc<dyn TaskHandle>, Error>,
        Request::MakeActive
    );

    pub async fn make_active(&self) -> Result<(), Error> {
        match self.make_active_raw().await?.wait().await {
            Ok(_) | Err(Error::UnitStarted) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub async fn update_def(&self, new: Service) -> Result<Service, Error> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(Request::UpdateDef(new, tx)).await.unwrap();
        rx.await.unwrap()
    }
}

struct Supervisor {
    receiver: mpsc::Receiver<Request>,
    current_task: CurrentTask,
    context: Arc<SupervisorContext>,
    timers: Box<Timers>,
}
impl Supervisor {
    pub fn start(mut self) {
        tokio::spawn(async move {
            self.timers = Timers::from(&self.context.service).into();
            self.run().await;
        });
    }

    pub async fn run(&mut self) -> Option<()> {
        loop {
            tokio::select! {
                req = self.receiver.recv() => self.handle_req(req?).await,
                Some(do_child) = do_child(&self.context, self.current_task.has_task()) => match do_child {
                    DoChild::Wait(wait) => self.handle_wait(wait).await,
                    DoChild::Stdout(msg) => self.log("stdout", &msg).await,
                    DoChild::Stderr(msg) => self.log("stderr", &msg).await,
                },
                Some(rslt) = self.current_task.wait() => self.handle_wait_task(rslt).await,
                Some(_) = Timers::recv(&mut self.timers.health_check) => self.handle_health_check().await,
            }
        }
    }

    /// Called when a request is sent to the supervisor.
    ///
    /// This function itself should return as soon as possible in order to prevent blocking the supervisor workflow. It handles
    /// requests, generates a response and immediately returns.
    async fn handle_req(&mut self, req: Request) {
        match req {
            Request::Query(chan) => {
                chan.send(self.query().await).ok();
            }
            Request::Start(chan) => {
                chan.send(self.user_start_service().await).ok();
            }
            Request::Stop(chan) => {
                chan.send(self.user_stop_service().await).ok();
            }
            Request::Reload(chan) => {
                chan.send(self.current_task.reload_service(self.context.clone()))
                    .ok();
            }
            Request::UpdateDef(new, chan) => {
                chan.send(self.update_def(new)).ok();
            }
            Request::GetTaskHandle(chan) => {
                chan.send(self.current_task.0.clone().ok_or(Error::TaskNotFound))
                    .ok();
            }
            Request::InterruptTask(chan) => {
                let handle = self.current_task.0.clone().ok_or(Error::TaskNotFound);
                if let Ok(x) = handle.as_deref() {
                    x.send_interrupt();
                }
                self.context.last_error.set_autosave(false);
                chan.send(handle).ok();
            }
            Request::MakeActive(chan) => {
                match &self.current_task.0 {
                    Some(task) => match task.task_class() {
                        "StartService" => chan.send(Ok(task.clone())).ok(),
                        _ => chan.send(Err(Error::TaskExists)).ok(),
                    },
                    None => match self.user_start_service().await {
                        Ok(handle) => chan.send(Ok(handle)).ok(),
                        Err(err) => chan.send(Err(err)).ok(),
                    },
                };
            }
        }
    }

    /// Called when the child process was terminated.
    ///
    /// This firstly sets the status of the service to `Stopped`. If retrying is enabled (`user_stop_service` is not called; even
    /// though `retry = 0`), it starts the "CleanupService" task, which may check if the service could be retried and (if it
    /// can), retry the service.
    async fn handle_wait(&mut self, wait: Wait) {
        self.context.status.set(Status::Stopped);
        self.context.set_child(None).await;
        if self.context.retry.enabled() {
            self.current_task
                .cleanup_service(self.context.clone(), wait)
                .ok();
        }
    }

    /// Called when current task finished.
    ///
    /// If error auto-saving is enabled, it sets last error to the task's result.
    async fn handle_wait_task(&mut self, rslt: Result<TaskFeedback, Error>) {
        if let Err(err) = rslt {
            if self.context.last_error.take_autosave() {
                self.context.last_error.set(err);
            }
        }
    }

    /// Called when the health check timer goes off.
    async fn handle_health_check(&mut self) {
        if self.context.status.get() == Status::Active {}
    }

    /// Called when the user attempted to start the service.
    ///
    /// This resets the retry counter, then returns the just-started "StartService" task if task creation succeeded.
    async fn user_start_service(&mut self) -> Result<Arc<dyn TaskHandle>, Error> {
        let would_interrupt = self
            .current_task
            .0
            .as_ref()
            .map(|x| !x.is_important())
            .unwrap_or_default();
        if would_interrupt {
            let task = self.current_task.0.take().unwrap();
            task.send_interrupt();
            task.wait().await.ok();
        }
        self.context.retry.reset();
        self.current_task.start_service(self.context.clone())
    }

    /// Called when the user attempted to stop the service.
    ///
    /// This disables retrying, then returns the just-started "StopService" task if task creation succeeded.
    async fn user_stop_service(&mut self) -> Result<Arc<dyn TaskHandle>, Error> {
        let would_interrupt = self
            .current_task
            .0
            .as_ref()
            .map(|x| !x.is_important())
            .unwrap_or_default();
        if would_interrupt {
            let task = self.current_task.0.take().unwrap();
            task.send_interrupt();
            task.wait().await.ok();
            return Ok(Arc::new(task::Empty));
        }
        self.context.retry.disable();
        self.current_task.stop_service(self.context.clone())
    }

    /// Updates service definition of the supervisor. On success, returns the elder service definition.
    ///
    /// # Errors
    /// This method would fail if the internal context has more than one reference, usually when a task is running for this
    /// supervisor.
    fn update_def(&mut self, new: Service) -> Result<Service, Error> {
        let context = Arc::get_mut(&mut self.context).ok_or(Error::TaskExists)?;
        if context.service != new {
            self.timers = Timers::from(&new).into();
        }

        Ok(std::mem::replace(&mut context.service, new))
    }

    /// Queries information about the supervisor.
    async fn query(&self) -> QueryService {
        let task = self.current_task.0.as_ref();
        QueryService {
            status: self.context.status.get(),
            status_since: Some(self.context.status.timestamp()),
            pid: self.context.pid().await.map(|x| x as _),
            task_class: task.map(|x| x.task_class().to_owned()),
            last_error: self.context.last_error.get(),
            definition: self.context.service.clone(),
        }
    }

    pub async fn log(&self, module: &str, msg: &[u8]) {
        airupd()
            .logger
            .write(
                &format!("airup_service_{}", self.context.service.name),
                module,
                msg,
            )
            .await
            .ok();
    }
}

#[derive(Default)]
struct CurrentTask(Option<Arc<dyn TaskHandle>>);
impl CurrentTask {
    fn has_task(&self) -> bool {
        self.0.is_some()
    }

    async fn wait(&mut self) -> Option<Result<TaskFeedback, Error>> {
        let val = self.0.as_deref()?.wait().await;
        self.0 = None;
        Some(val)
    }

    start_task!(start_service, StartServiceHandle);
    start_task!(stop_service, StopServiceHandle);
    start_task!(reload_service, ReloadServiceHandle);

    fn cleanup_service(
        &mut self,
        context: Arc<SupervisorContext>,
        wait: Wait,
    ) -> Result<Arc<dyn TaskHandle>, Error> {
        self._start_task(context, |ctx| CleanupServiceHandle::new(ctx, wait))
    }

    fn _start_task<F: FnOnce(Arc<SupervisorContext>) -> Arc<dyn TaskHandle>>(
        &mut self,
        context: Arc<SupervisorContext>,
        task_new: F,
    ) -> Result<Arc<dyn TaskHandle>, Error> {
        if self.0.is_some() {
            return Err(Error::TaskExists);
        }
        context.last_error.set_autosave(false);
        let task = task_new(context.clone());
        self.0 = Some(task);
        Ok(self.0.as_ref().cloned().unwrap())
    }
}

/// Context of a running supervisor.
#[derive(Debug)]
pub struct SupervisorContext {
    pub service: Service,
    pub last_error: LastErrorContext,
    pub status: StatusContext,
    child: tokio::sync::RwLock<Option<Child>>,
    retry: RetryContext,
}
impl SupervisorContext {
    /// Creates a new [`SupervisorContext`] instance for the given [`Service`].
    pub fn new(service: Service) -> Arc<Self> {
        Arc::new(Self {
            service,
            last_error: Default::default(),
            status: Default::default(),
            child: Default::default(),
            retry: Default::default(),
        })
    }

    /// Returns main PID of the service supervised by the supervisor.
    pub async fn pid(&self) -> Option<i64> {
        self.child.read().await.as_ref().map(|x| x.id())
    }

    /// Sets new child for the supervisor.
    pub async fn set_child<T: Into<Option<Child>>>(&self, new: T) {
        *self.child.write().await = new.into();
    }
}

#[derive(Debug, Default)]
pub struct StatusContext {
    data: Mutex<Status>,
    timestamp: AtomicI64,
}
impl StatusContext {
    /// Gets current status.
    pub fn get(&self) -> Status {
        *self.data.lock().unwrap()
    }

    /// Gets the timestamp of last status change.
    pub fn timestamp(&self) -> i64 {
        self.timestamp.load(atomic::Ordering::Acquire)
    }

    /// Changes current status updating timestamp.
    pub fn set(&self, new: Status) -> Status {
        let mut lock = self.data.lock().unwrap();
        self.timestamp
            .store(airupfx::time::timestamp_ms(), atomic::Ordering::Release);
        std::mem::replace(&mut lock, new)
    }
}

#[derive(Debug, Default)]
pub struct LastErrorContext {
    data: RwLock<Option<Error>>,
    auto_save: AtomicBool,
}
impl LastErrorContext {
    pub fn set<E: Into<Option<Error>>>(&self, new: E) -> Option<Error> {
        std::mem::replace(&mut self.data.write().unwrap(), new.into())
    }

    pub fn get(&self) -> Option<Error> {
        self.data.read().unwrap().clone()
    }

    pub fn set_autosave(&self, val: bool) -> bool {
        self.auto_save.swap(val, atomic::Ordering::SeqCst)
    }

    pub fn take_autosave(&self) -> bool {
        self.set_autosave(false)
    }
}

#[derive(Debug, Default)]
pub struct RetryContext {
    disabled: AtomicBool,
    count: AtomicI32,
}
impl RetryContext {
    /// Returns `true` if the service should be retried, then increases retry counter.
    pub fn check_and_mark(&self, max: i32) -> bool {
        if self.disabled() {
            return false;
        }

        match max {
            -1 => true,
            max => self
                .count
                .fetch_update(
                    atomic::Ordering::SeqCst,
                    atomic::Ordering::SeqCst,
                    |x| match (x + 1).cmp(&max) {
                        cmp::Ordering::Less => Some(x + 1),
                        cmp::Ordering::Equal | cmp::Ordering::Greater => None,
                    },
                )
                .is_ok(),
        }
    }

    /// Resets the retry counter.
    ///
    /// If retrying is disabled, it will be enabled. Then the retry count is set to zero.
    pub fn reset(&self) {
        self.enable();
        self.reset_count();
    }

    /// Sets the retry count to zero.
    pub fn reset_count(&self) {
        self.count.store(0, atomic::Ordering::SeqCst);
    }

    /// Enables retrying if disabled.
    pub fn enable(&self) {
        self.disabled.store(false, atomic::Ordering::Release);
    }

    /// Disables retrying if enabled.
    pub fn disable(&self) {
        self.disabled.store(true, atomic::Ordering::Release);
    }

    /// Returns `true` if retrying is enabled.
    pub fn enabled(&self) -> bool {
        !self.disabled()
    }

    /// Returns `true` if retrying is disabled.
    pub fn disabled(&self) -> bool {
        self.disabled.load(atomic::Ordering::Acquire)
    }
}

#[derive(Debug, Default)]
struct Timers {
    health_check: Option<Timer>,
}
impl From<&Service> for Timers {
    fn from(service: &Service) -> Self {
        let mut result = Self::default();

        if matches!(service.watchdog.kind, Some(WatchdogKind::HealthCheck)) {
            result.health_check = Some(Timer::new(Duration::from_millis(
                service.watchdog.health_check_interval,
            )));
        }

        result
    }
}
impl Timers {
    #[allow(clippy::unit_arg)]
    async fn recv(timer: &mut Option<Timer>) -> Option<()> {
        match timer {
            Some(x) => Some(x.recv().await),
            None => None,
        }
    }
}

#[derive(Debug)]
struct Timer {
    abort_handle: AbortHandle,
    rx: mpsc::Receiver<()>,
}
impl Timer {
    fn new(dur: Duration) -> Self {
        let (tx, rx) = mpsc::channel(1);
        let join_handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(dur).await;
                let Ok(_) = tx.send(()).await else {
                    break;
                };
            }
        });

        Self {
            abort_handle: join_handle.abort_handle(),
            rx,
        }
    }

    async fn recv(&mut self) {
        self.rx.recv().await.unwrap()
    }
}
impl Drop for Timer {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}

impl crate::app::Airupd {
    /// Waits until the specific service is active.
    ///
    /// If the specific service is already active, the method immediately returns. If the service is being started, it waits
    /// until the running `StartService` task is done. If the service is stopped, it attempts to start the service and waits the
    /// task to finish.
    ///
    /// # Errors
    /// This method would fail if the service is running a task but is not `StartService` or the specific service was not found.
    pub async fn make_service_active(&self, name: &str) -> Result<(), Error> {
        let supervisor = match self.supervisors.get(name).await {
            Some(supervisor) => supervisor,
            None => {
                self.supervisors
                    .supervise(self.storage.get_service_patched(name).await?)
                    .await
            }
        };
        supervisor.make_active().await
    }

    /// Starts the specific service, returns a handle of the spawned `StartService` task on success.
    ///
    /// # Errors
    /// This method would fail if the service is already active, having another running task or the specific service was not
    /// found.
    pub async fn start_service(&self, name: &str) -> Result<Arc<dyn TaskHandle>, Error> {
        match self.supervisors.get(name).await {
            Some(supervisor) => Ok(supervisor.start().await?),
            None => {
                let supervisor = self
                    .supervisors
                    .supervise(self.storage.get_service_patched(name).await?)
                    .await;
                Ok(supervisor.start().await?)
            }
        }
    }

    /// Queries the specific service, returns queried information about the service.
    ///
    /// # Errors
    /// This method would fail if the specific service was not found.
    pub async fn query_service(&self, name: &str) -> Result<QueryService, Error> {
        match self.supervisors.get(name).await {
            Some(supervisor) => Ok(supervisor.query().await),
            None => Ok(QueryService::default_of(
                self.storage.get_service_patched(name).await?,
            )),
        }
    }

    /// Stops the specific service, returns a handle of the spawned `StopService` task on success.
    ///
    /// # Errors
    /// This method would fail if the service is not active, having another running task or the specific service was not
    /// found.
    pub async fn stop_service(&self, name: &str) -> Result<Arc<dyn TaskHandle>, Error> {
        match self.supervisors.get(name).await {
            Some(supervisor) => Ok(supervisor.stop().await?),
            None => {
                self.storage.get_service_patched(name).await?;
                Err(Error::UnitNotStarted)
            }
        }
    }

    /// Reloads the specific service, returns a handle of the spawned `ReloadService` task on success.
    ///
    /// # Errors
    /// This method would fail if the service is not active, having another running task or the specific service was not
    /// found.
    pub async fn reload_service(&self, name: &str) -> Result<Arc<dyn TaskHandle>, Error> {
        match self.supervisors.get(name).await {
            Some(supervisor) => Ok(supervisor.reload().await?),
            None => {
                self.storage.get_service_patched(name).await?;
                Err(Error::UnitNotStarted)
            }
        }
    }

    /// Caches the specific service.
    ///
    /// # Errors
    /// This method would fail if the service was not found.
    pub async fn cache_service(&self, name: &str) -> Result<(), Error> {
        self.supervisors
            .supervise(self.storage.get_service_patched(name).await?)
            .await;
        Ok(())
    }

    /// Removes the specific service from cache.
    ///
    /// # Errors
    /// This method would fail if the service was not previously cached.
    pub async fn uncache_service(&self, name: &str) -> Result<(), Error> {
        self.supervisors.remove(name).await
    }

    /// Interrupts current running task of the specific service, returns a handle of the task.
    ///
    /// # Errors
    /// This method would fail if the service has no running task.
    pub async fn interrupt_service_task(&self, name: &str) -> Result<Arc<dyn TaskHandle>, Error> {
        let name = name.strip_suffix(".airs").unwrap_or(name);
        match self.supervisors.get(name).await {
            Some(supervisor) => Ok(supervisor.interrupt_task().await?),
            None => {
                self.storage.get_service_patched(name).await?;
                Err(Error::UnitNotStarted)
            }
        }
    }
}

/// Representation of a request sent to a supervisor.
enum Request {
    Query(oneshot::Sender<QueryService>),
    Start(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
    Stop(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
    Reload(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
    UpdateDef(Service, oneshot::Sender<Result<Service, Error>>),
    GetTaskHandle(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
    InterruptTask(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
    MakeActive(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
}

enum DoChild {
    Wait(Wait),
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
}

async fn do_child(context: &SupervisorContext, has_task: bool) -> Option<DoChild> {
    let mut lock = context.child.write().await;

    let stdout = lock.as_ref().and_then(|x| x.stdout());
    let stderr = lock.as_ref().and_then(|x| x.stderr());

    if has_task || lock.is_none() {
        return None;
    }

    tokio::select! {
        Some(wait) = wait(&mut lock) => Some(DoChild::Wait(wait)),
        Some(line) = wait_piper(stdout) => Some(DoChild::Stdout(line)),
        Some(line) = wait_piper(stderr) => Some(DoChild::Stderr(line)),
    }
}

async fn wait(lock: &mut Option<Child>) -> Option<Wait> {
    debug_assert!(lock.is_some());
    let wait = lock.as_mut().unwrap().wait().await.ok();
    *lock = None;
    wait
}

async fn wait_piper(piper: Option<Arc<PiperHandle>>) -> Option<Vec<u8>> {
    piper?.read_line().await
}
