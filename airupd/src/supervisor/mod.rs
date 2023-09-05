mod app_integration;
pub mod task;

pub use app_integration::AirupdExt;

use self::task::{
    CleanupServiceHandle, ReloadServiceHandle, StartServiceHandle, StopServiceHandle, TaskFeedback,
    TaskHandle,
};
use airupfx::{
    ace::Child,
    files::Service,
    prelude::*,
    process::Wait,
    sdk::{
        system::{QueryResult, Status},
        Error,
    },
};
use smallvec::{smallvec, SmallVec};
use std::{
    cmp,
    collections::HashMap,
    sync::{
        atomic::{self, AtomicBool, AtomicI32},
        Arc, Mutex, RwLock,
    },
};
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
    supervisors: RwLock<HashMap<String, Arc<SupervisorHandle>>>,
}
impl Manager {
    /// Creates a new, empty [Manager] instance.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Supervises the given service in the supervisor set. Returns `(false, _)` if the supervisor already exists.
    pub async fn supervise(&self, service: Service) -> Result<Arc<SupervisorHandle>, Error> {
        let mut lock = self.supervisors.write().unwrap();

        if let Some(x) = lock.get(&service.name).cloned() {
            return Ok(x);
        }

        let name = service.name.clone();

        let handle = SupervisorHandle::new(service);
        lock.insert(name, handle.clone());

        Ok(handle)
    }

    /// Gets a supervisor in the set.
    pub fn get(&self, name: &str) -> Option<Arc<SupervisorHandle>> {
        self.supervisors.read().unwrap().get(name).cloned()
    }

    /// Safely removes a supervisor from the set.
    pub async fn quit(&self, name: &str) -> Result<(), Error> {
        self.supervisors
            .write()
            .unwrap()
            .remove(name)
            .map(|_| ())
            .ok_or(Error::ObjectNotFound)
    }

    /// Gets a list of names of supervisors.
    pub fn list(&self) -> Vec<String> {
        self.supervisors
            .read()
            .unwrap()
            .keys()
            .map(|x| x.into())
            .collect()
    }
}

/// Handle of an Airup supervisor.
#[derive(Debug)]
pub struct SupervisorHandle {
    sender: mpsc::Sender<Request>,
}
impl SupervisorHandle {
    /// Creates a new [SupervisorHandle] instance while starting the associated supervisor.
    pub fn new(service: Service) -> Arc<Self> {
        let (sender, receiver) = mpsc::channel(128);

        let supervisor = Supervisor {
            receiver,
            current_task: CurrentTask::default(),
            context: SupervisorContext::new(service),
        };
        supervisor.start();

        Arc::new(Self { sender })
    }

    supervisor_req!(query, QueryResult, Request::Query);
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

    /// If the service is running for `StartService` task, waits until the task completed. Otherwise it will attepmt to start
    /// the service and waits until it completed.
    ///
    /// ## Cancel Safety
    /// This method is cancel safe.
    pub async fn make_active(&self) -> Result<(), Error> {
        let (task_handle_tx, task_handle_rx) = oneshot::channel();
        let (start_tx, start_rx) = oneshot::channel();
        self.sender
            .send(Request::Transaction(vec![
                Request::GetTaskHandle(task_handle_tx),
                Request::Start(start_tx),
            ]))
            .await
            .unwrap();

        let task_handle = task_handle_rx.await.unwrap();
        let start = start_rx.await.unwrap();

        if let Ok(task_handle) = task_handle {
            if task_handle.task_type() == "StartService" {
                return task_handle.wait().await.map(|_| ());
            } else {
                return Err(Error::TaskAlreadyExists);
            }
        }

        match start.unwrap().wait().await {
            Ok(_) | Err(Error::ObjectAlreadyConfigured) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

struct Supervisor {
    receiver: mpsc::Receiver<Request>,
    current_task: CurrentTask,
    context: Arc<SupervisorContext>,
}
impl Supervisor {
    pub fn start(mut self) {
        tokio::spawn(async move {
            self.run().await;
        });
    }

    pub async fn run(&mut self) -> Option<()> {
        loop {
            tokio::select! {
                req = self.receiver.recv() => self.handle_req(req?).await,
                Some(wait) = wait(&self.context, self.current_task.has_task()) => self.handle_wait(wait).await,
                Some(rslt) = self.current_task.wait() => self.handle_wait_task(rslt).await,
            }
        }
    }

    async fn handle_req(&mut self, req: Request) {
        let mut transaction: SmallVec<[_; 1]> = smallvec![req];
        loop {
            if transaction.is_empty() {
                break;
            }

            let drained: SmallVec<[_; 1]> = transaction.drain(..).collect();
            for req in drained {
                match req {
                    Request::Query(chan) => {
                        let query_result = QueryResult {
                            status: self.context.status.get(),
                            pid: self.context.pid().await,
                            task: self
                                .current_task
                                .0
                                .as_ref()
                                .map(|x| x.task_type().to_owned()),
                            last_error: self.context.last_error.get(),
                            service: self.context.service.clone(),
                        };
                        chan.send(query_result).ok();
                    }
                    Request::Start(chan) => {
                        self.context.retry.reset();
                        chan.send(self.current_task.start_service(self.context.clone()))
                            .ok();
                    }
                    Request::Stop(chan) => {
                        self.context.retry.disable();
                        chan.send(self.current_task.stop_service(self.context.clone()))
                            .ok();
                    }
                    Request::Reload(chan) => {
                        chan.send(self.current_task.reload_service(self.context.clone()))
                            .ok();
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
                    Request::Transaction(list) => {
                        transaction.append::<[Request; 0]>(&mut list.into());
                    }
                }
            }
        }
    }

    async fn handle_wait(&mut self, wait: Wait) {
        self.context.status.set(Status::Stopped);
        self.context.set_child(None).await;
        if self.context.retry.enabled() {
            self.current_task
                .cleanup_service(self.context.clone(), wait)
                .ok();
        }
    }

    async fn handle_wait_task(&mut self, rslt: Result<TaskFeedback, Error>) {
        if let Err(err) = rslt {
            if self.context.last_error.take_autosave() {
                self.context.last_error.set(err);
            }
        }
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

    fn cleanup_service(
        &mut self,
        context: Arc<SupervisorContext>,
        wait: Wait,
    ) -> Result<Arc<dyn TaskHandle>, Error> {
        self._start_task(context, |ctx| CleanupServiceHandle::new(ctx, wait))
    }

    fn start_service(
        &mut self,
        context: Arc<SupervisorContext>,
    ) -> Result<Arc<dyn TaskHandle>, Error> {
        self._start_task(context, StartServiceHandle::new)
    }

    fn stop_service(
        &mut self,
        context: Arc<SupervisorContext>,
    ) -> Result<Arc<dyn TaskHandle>, Error> {
        self._start_task(context, StopServiceHandle::new)
    }

    fn reload_service(
        &mut self,
        context: Arc<SupervisorContext>,
    ) -> Result<Arc<dyn TaskHandle>, Error> {
        self._start_task(context, ReloadServiceHandle::new)
    }

    fn _start_task<F: FnOnce(Arc<SupervisorContext>) -> Arc<dyn TaskHandle>>(
        &mut self,
        context: Arc<SupervisorContext>,
        task_new: F,
    ) -> Result<Arc<dyn TaskHandle>, Error> {
        if self.0.is_some() {
            return Err(Error::TaskAlreadyExists);
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
    /// Creates a new [SupervisorContext] instance for the given [Service].
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
    pub async fn pid(&self) -> Option<Pid> {
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
}
impl StatusContext {
    pub fn get(&self) -> Status {
        *self.data.lock().unwrap()
    }

    pub fn set(&self, new: Status) -> Status {
        std::mem::replace(&mut *self.data.lock().unwrap(), new)
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

    pub fn reset(&self) {
        self.enable();
        self.reset_count();
    }

    pub fn reset_count(&self) {
        self.count.store(0, atomic::Ordering::Relaxed);
    }

    pub fn enable(&self) {
        self.disabled.store(false, atomic::Ordering::Relaxed);
    }

    pub fn disable(&self) {
        self.disabled.store(true, atomic::Ordering::Relaxed);
    }

    pub fn enabled(&self) -> bool {
        !self.disabled()
    }

    pub fn disabled(&self) -> bool {
        self.disabled.load(atomic::Ordering::Relaxed)
    }
}

enum Request {
    Query(oneshot::Sender<QueryResult>),
    Start(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
    Stop(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
    Reload(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
    GetTaskHandle(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
    InterruptTask(oneshot::Sender<Result<Arc<dyn TaskHandle>, Error>>),
    Transaction(Vec<Self>),
}

async fn wait(context: &SupervisorContext, has_task: bool) -> Option<Wait> {
    if has_task {
        return None;
    }

    let mut lock = context.child.write().await;
    let wait = match lock.as_mut() {
        Some(x) => x.wait().await.ok(),
        None => None,
    };

    if wait.is_some() {
        *lock = None;
    }

    wait
}
