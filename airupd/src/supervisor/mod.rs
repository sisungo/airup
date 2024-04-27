//! # The Airup Supervisor
//! Main module containing full airup supervisor logic.

pub mod task;

use crate::{ace::Child, app::airupd};
use airup_sdk::{
    files::{service::WatchdogKind, Service},
    system::{Event, QueryService, Status},
    Error,
};
use airupfx::{isolator::Realm, process::Wait, time::Alarm};
use std::{
    cmp,
    collections::HashMap,
    sync::{
        atomic::{self, AtomicBool, AtomicI32, AtomicI64},
        Arc, Mutex, RwLock,
    },
    time::Duration,
};
use task::{Empty, TaskHandle};
use tokio::sync::{mpsc, oneshot};

macro_rules! supervisor_req {
    ($name:ident, $ret:ty, $req:expr) => {
        pub async fn $name(&self) -> $ret {
            let (tx, rx) = oneshot::channel();
            self.sender.send($req(tx)).await.unwrap();
            rx.await.unwrap()
        }
    };
}

/// A manager of Airup supervisors.
#[derive(Debug, Default)]
pub struct Manager {
    supervisors: tokio::sync::RwLock<HashMap<String, Arc<SupervisorHandle>>>,
    provided: tokio::sync::RwLock<HashMap<String, Arc<SupervisorHandle>>>,
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
        supervisors: &mut HashMap<String, Arc<SupervisorHandle>>,
        provided: &mut HashMap<String, Arc<SupervisorHandle>>,
        permissive: bool,
    ) -> Result<(), Error> {
        let handle = supervisors.get(name).ok_or(Error::NotStarted)?.clone();
        let queried = handle.query().await;

        let is_providing = |provided: &mut HashMap<_, _>, i| {
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
            Err(Error::Started)
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
            let new = airupd()
                .storage
                .get_service_patched(&queried.definition.name)
                .await;
            let new = match new {
                Ok(x) => x,
                Err(_) => {
                    errors.push(k.into());
                    continue;
                }
            };
            if v.update_def(Box::new(new)).await.is_err() {
                errors.push(k.into());
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
            events: airupd().events.subscribe(),
        };
        supervisor.start();

        Arc::new(Self { sender })
    }

    supervisor_req!(query, QueryService, Request::Query);
    supervisor_req!(start, Result<Arc<dyn TaskHandle>, Error>, Request::Start);
    supervisor_req!(stop, Result<Arc<dyn TaskHandle>, Error>, Request::Stop);
    supervisor_req!(kill, Result<(), Error>, Request::Kill);
    supervisor_req!(reload, Result<Arc<dyn TaskHandle>, Error>, Request::Reload);
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
            Ok(_) | Err(Error::Started) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub async fn update_def(&self, new: Box<Service>) -> Result<Service, Error> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(Request::UpdateDef(new, tx)).await.unwrap();
        rx.await.unwrap()
    }
}

/// A supervisor entity.
struct Supervisor {
    receiver: mpsc::Receiver<Request>,
    current_task: CurrentTask,
    context: Arc<SupervisorContext>,
    timers: Box<Timers>,
    events: async_broadcast::Receiver<Event>,
}
impl Supervisor {
    /// Starts the supervisor task.
    fn start(mut self) {
        tokio::spawn(async move {
            self.timers = Timers::from(&self.context.service).into();
            self.run().await;
        });
    }

    /// Main logic of the supervisor task.
    async fn run(&mut self) -> Option<()> {
        loop {
            tokio::select! {
                req = self.receiver.recv() => self.handle_req(req?).await,
                Some(wait) = do_child(&self.context, self.current_task.has_task()) => self.handle_wait(wait).await,
                Some(handle) = self.current_task.wait() => self.handle_wait_task(handle).await,
                Some(_) = Timers::wait(&mut self.timers.health_check) => self.handle_health_check().await,
                Ok(event) = self.events.recv() => self.handle_event(&event).await,
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
                chan.send(self.user_stop_service(false).await).ok();
            }
            Request::Kill(chan) => {
                chan.send(self.user_stop_service(true).await.map(|_| ()))
                    .ok();
            }
            Request::Reload(chan) => {
                chan.send(self.reload_service().await).ok();
            }
            Request::UpdateDef(new, chan) => {
                chan.send(self.update_def(*new).await).ok();
            }
            Request::InterruptTask(chan) => {
                let handle = self.current_task.interrupt();
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

    /// Called when an event is triggered.
    async fn handle_event(&mut self, event: &Event) {
        if let Some(exec) = self.context.service.event_handlers.get(&event.id) {
            let Ok(mut ace) = task::ace(&self.context).await else {
                return;
            };
            ace.env.var("AIRUP_EVENT_HANDLER_PAYLOAD", &event.payload);
            ace.run(exec).await.ok();
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
            self.cleanup_service(wait).await.ok();
        }
    }

    /// Called when current task finished.
    ///
    /// If error auto-saving is enabled, it sets last error to the task's result.
    async fn handle_wait_task(&mut self, handle: Arc<dyn TaskHandle>) {
        let Err(error) = handle.wait().await else {
            return;
        };
        if handle.task_class() == "HealthCheck" {
            self.context.last_error.set(Error::Watchdog);
            self.stop_service().await.ok();
        }
        if self.context.last_error.take_autosave() {
            self.context.last_error.set(error);
        }
    }

    /// Called when the health check timer goes off.
    async fn handle_health_check(&mut self) {
        if self.context.status.get() == Status::Active {
            self.health_check().await.ok();
        } else if let Some(alarm) = &mut self.timers.health_check {
            alarm.disable();
        }
    }

    /// Queries information about the supervisor.
    async fn query(&self) -> QueryService {
        let task = self.current_task.0.as_ref();
        QueryService {
            status: self.context.status.get(),
            status_since: Some(self.context.status.timestamp()),
            pid: self.context.pid().await.map(|x| x as _),
            memory_usage: self
                .context
                .realm
                .as_ref()
                .and_then(|x| x.memory_usage().ok())
                .map(|x| x as u64),
            task_class: task.map(|x| x.task_class().to_owned()),
            last_error: self.context.last_error.get(),
            definition: self.context.service.clone(),
        }
    }

    /// Updates service definition of the supervisor. On success, returns the elder service definition.
    ///
    /// # Errors
    /// This method would fail if the internal context has more than one reference, usually when a task is running for this
    /// supervisor.
    async fn update_def(&mut self, new: Service) -> Result<Service, Error> {
        self.current_task.interrupt_non_important().await;
        let context = Arc::get_mut(&mut self.context).ok_or(Error::TaskExists)?;
        if context.service != new {
            self.timers = Timers::from(&new).into();
        }
        setup_realm(&context.realm, &new);

        Ok(std::mem::replace(&mut context.service, new))
    }

    /// Called when the user attempted to start the service.
    ///
    /// This resets the retry counter, then returns the just-started "StartService" task if task creation succeeded.
    async fn user_start_service(&mut self) -> Result<Arc<dyn TaskHandle>, Error> {
        self.current_task.interrupt_non_important().await;
        self.context.retry.reset();
        self.start_service().await
    }

    /// Called when the user attempted to stop the service.
    ///
    /// This disables retrying, then returns the just-started "StopService" task if task creation succeeded.
    async fn user_stop_service(&mut self, force: bool) -> Result<Arc<dyn TaskHandle>, Error> {
        if self.current_task.interrupt_non_important().await {
            return Ok(Arc::new(task::Empty));
        }
        self.context.retry.disable();
        match force {
            true => self.kill_service().await,
            false => self.stop_service().await,
        }
    }

    /// Starts the service.
    async fn start_service(&mut self) -> Result<Arc<dyn TaskHandle>, Error> {
        self.timers.on_start();
        self.current_task
            ._start_task(&self.context, async {
                task::start::start(self.context.clone())
            })
            .await
    }

    /// Stops the service.
    async fn stop_service(&mut self) -> Result<Arc<dyn TaskHandle>, Error> {
        self.current_task
            ._start_task(&self.context, async {
                task::stop::start(self.context.clone())
            })
            .await
    }

    /// Forces the service to stop.
    async fn kill_service(&mut self) -> Result<Arc<dyn TaskHandle>, Error> {
        let child = self.context.child.read().await;

        if let Some(realm) = &self.context.realm {
            if realm.kill().is_ok() {
                return Ok(Arc::new(Empty));
            }
        }

        if let Some(ch) = &*child {
            ch.kill().await?;
            Ok(Arc::new(Empty))
        } else {
            Err(Error::unsupported(
                "cannot kill a service without a process",
            ))
        }
    }

    /// Reloads the service.
    async fn reload_service(&mut self) -> Result<Arc<dyn TaskHandle>, Error> {
        self.current_task
            ._start_task(&self.context, task::reload::start(&self.context))
            .await
    }

    /// Cleans the service up.
    async fn cleanup_service(&mut self, wait: Wait) -> Result<Arc<dyn TaskHandle>, Error> {
        self.current_task
            ._start_task(&self.context, async {
                task::cleanup::start(self.context.clone(), wait)
            })
            .await
    }

    /// Executes health check for the service.
    async fn health_check(&mut self) -> Result<Arc<dyn TaskHandle>, Error> {
        self.current_task
            ._start_task(&self.context, async {
                task::health_check::start(&self.context).await
            })
            .await
    }
}

/// A container of current running task in the supervisor.
#[derive(Default)]
struct CurrentTask(Option<Arc<dyn TaskHandle>>);
impl CurrentTask {
    /// Returns `true` if a task is running in the container.
    fn has_task(&self) -> bool {
        self.0.is_some()
    }

    /// Waits until the task running in the container to be finished. When the operation completed, a `Some(_)` is returned
    /// and the container is set empty. If the container is already empty, `None` is returned.
    async fn wait(&mut self) -> Option<Arc<dyn TaskHandle>> {
        self.0.as_deref()?.wait().await.ok();
        self.0.take()
    }

    /// Interrupts and waits for the running task if it is tagged "non-important". Returns `true` if a task is interrupted.
    async fn interrupt_non_important(&mut self) -> bool {
        let would_interrupt = self
            .0
            .as_ref()
            .map(|x| !x.is_important())
            .unwrap_or_default();
        if would_interrupt {
            let task = self.0.take().unwrap();
            task.send_interrupt();
            task.wait().await.ok();
        }

        would_interrupt
    }

    /// Sends interrupt to the running task.
    fn interrupt(&mut self) -> Result<Arc<dyn TaskHandle>, Error> {
        let handle = self.0.clone().ok_or(Error::TaskNotFound);
        if let Ok(x) = handle.as_deref() {
            x.send_interrupt();
        }
        handle
    }

    /// Starts a task in the container.
    ///
    /// # Errors
    /// The following errors would be returned by calling this function:
    ///  - [`Error::TaskExists`]: A task is already running in this container.
    async fn _start_task(
        &mut self,
        context: &SupervisorContext,
        task_new: impl std::future::Future<Output = Arc<dyn TaskHandle>>,
    ) -> Result<Arc<dyn TaskHandle>, Error> {
        if self.0.is_some() {
            return Err(Error::TaskExists);
        }
        context.last_error.set_autosave(false);
        let task = task_new.await;
        self.0 = Some(task);
        Ok(self.0.as_ref().cloned().unwrap())
    }
}

/// Context of a running supervisor.
#[derive(Debug)]
struct SupervisorContext {
    service: Service,
    last_error: LastErrorContext,
    status: StatusContext,
    realm: Option<Arc<Realm>>,
    child: tokio::sync::RwLock<Option<Child>>,
    retry: RetryContext,
}
impl SupervisorContext {
    /// Creates a new [`SupervisorContext`] instance for the given [`Service`].
    fn new(service: Service) -> Arc<Self> {
        let realm = Realm::new().ok().map(Arc::new);
        setup_realm(&realm, &service);
        Arc::new(Self {
            service,
            last_error: Default::default(),
            status: Default::default(),
            realm,
            child: Default::default(),
            retry: Default::default(),
        })
    }

    /// Returns main PID of the service supervised by the supervisor.
    async fn pid(&self) -> Option<i64> {
        self.child.read().await.as_ref().map(|x| x.id())
    }

    /// Sets new child for the supervisor.
    async fn set_child<T: Into<Option<Child>>>(&self, new: T) -> Option<Child> {
        std::mem::replace(&mut *self.child.write().await, new.into())
    }
}

#[derive(Debug, Default)]
struct StatusContext {
    data: Mutex<Status>,
    timestamp: AtomicI64,
}
impl StatusContext {
    /// Gets current status.
    fn get(&self) -> Status {
        *self.data.lock().unwrap()
    }

    /// Gets the timestamp of last status change.
    fn timestamp(&self) -> i64 {
        self.timestamp.load(atomic::Ordering::Acquire)
    }

    /// Changes current status updating timestamp.
    fn set(&self, new: Status) {
        let mut lock = self.data.lock().unwrap();
        self.timestamp
            .store(airupfx::time::timestamp_ms(), atomic::Ordering::Release);
        *lock = new;
    }
}

#[derive(Debug, Default)]
struct LastErrorContext {
    data: RwLock<Option<Error>>,
    auto_save: AtomicBool,
}
impl LastErrorContext {
    fn set<E: Into<Option<Error>>>(&self, new: E) -> Option<Error> {
        std::mem::replace(&mut self.data.write().unwrap(), new.into())
    }

    fn get(&self) -> Option<Error> {
        self.data.read().unwrap().clone()
    }

    fn set_autosave(&self, val: bool) -> bool {
        self.auto_save.swap(val, atomic::Ordering::SeqCst)
    }

    fn take_autosave(&self) -> bool {
        self.set_autosave(false)
    }
}

#[derive(Debug, Default)]
struct RetryContext {
    disabled: AtomicBool,
    count: AtomicI32,
}
impl RetryContext {
    /// Returns `true` if the service should be retried, then increases retry counter.
    fn check_and_mark(&self, max: i32) -> bool {
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
    fn reset(&self) {
        self.enable();
        self.reset_count();
    }

    /// Sets the retry count to zero.
    fn reset_count(&self) {
        self.count.store(0, atomic::Ordering::SeqCst);
    }

    /// Enables retrying if disabled.
    fn enable(&self) {
        self.disabled.store(false, atomic::Ordering::Release);
    }

    /// Disables retrying if enabled.
    fn disable(&self) {
        self.disabled.store(true, atomic::Ordering::Release);
    }

    /// Returns `true` if retrying is enabled.
    fn enabled(&self) -> bool {
        !self.disabled()
    }

    /// Returns `true` if retrying is disabled.
    fn disabled(&self) -> bool {
        self.disabled.load(atomic::Ordering::Acquire)
    }
}

#[derive(Debug, Default)]
struct Timers {
    health_check: Option<Alarm>,
}
impl From<&Service> for Timers {
    fn from(service: &Service) -> Self {
        let mut result = Self::default();

        if matches!(service.watchdog.kind, Some(WatchdogKind::HealthCheck)) {
            result.health_check = Some(Alarm::new(Duration::from_millis(
                service.watchdog.health_check_interval as _,
            )));
        }

        result
    }
}
impl Timers {
    async fn wait(timer: &mut Option<Alarm>) -> Option<()> {
        match timer {
            Some(x) => x.wait().await,
            None => None,
        }
    }

    fn on_start(&mut self) {
        if let Some(alarm) = &mut self.health_check {
            alarm.enable();
        }
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
                Err(Error::NotStarted)
            }
        }
    }

    /// Forces the specific service to stop.
    ///
    /// # Errors
    /// This method would fail if the service does not have a process.
    pub async fn kill_service(&self, name: &str) -> Result<(), Error> {
        match self.supervisors.get(name).await {
            Some(supervisor) => Ok(supervisor.kill().await?),
            None => {
                self.storage.get_service_patched(name).await?;
                Err(Error::NotStarted)
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
                Err(Error::NotStarted)
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
                Err(Error::NotStarted)
            }
        }
    }
}

/// Representation of a request sent to a supervisor.
enum Request {
    Query(oneshot::Sender<QueryService>),
    Start(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
    Stop(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
    Kill(oneshot::Sender<Result<(), Error>>),
    Reload(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
    UpdateDef(Box<Service>, oneshot::Sender<Result<Service, Error>>),
    InterruptTask(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
    MakeActive(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
}

async fn do_child(context: &SupervisorContext, has_task: bool) -> Option<Wait> {
    let mut lock = context.child.write().await;

    if has_task || lock.is_none() {
        return None;
    }

    wait(&mut lock).await
}

async fn wait(lock: &mut Option<Child>) -> Option<Wait> {
    debug_assert!(lock.is_some());
    let wait = lock.as_mut().unwrap().wait().await.ok();
    *lock = None;
    wait
}

fn setup_realm(realm: &Option<Arc<Realm>>, service: &Service) {
    if let Some(realm) = &realm {
        if let Some(x) = service.reslimit.cpu {
            realm.set_cpu_limit(x).ok();
        }

        if let Some(x) = service.reslimit.memory {
            realm.set_mem_limit(x as usize).ok();
        }
    }
}
