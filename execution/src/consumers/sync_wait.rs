use crate::traits::{OperationState, Receiver, Sender};
use std::hint::unreachable_unchecked;
use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug)]
pub enum WaitResult<V> {
    Value(V),
    Error(crate::Error),
    Stopped,
}

impl<V> WaitResult<V> {
    pub fn unwrap(self) -> V {
        match self {
            WaitResult::Value(v) => v,
            _ => panic!("Result does not contain a value"),
        }
    }

    pub fn unwrap_error(self) -> crate::Error {
        match self {
            WaitResult::Error(v) => v,
            _ => panic!("Result does not contain an error"),
        }
    }

    pub fn unwrap_stopped(self) {
        match self {
            WaitResult::Stopped => {}
            _ => panic!("Result is not stopped!"),
        }
    }

    pub fn into_result(self) -> crate::Result<V> {
        match self {
            WaitResult::Value(v) => Ok(Some(v)),
            WaitResult::Error(e) => Err(e),
            WaitResult::Stopped => Ok(None),
        }
    }

    pub fn is_value(&self) -> bool {
        matches!(self, WaitResult::Value(_))
    }

    pub fn is_stopped(&self) -> bool {
        matches!(self, WaitResult::Stopped)
    }
}

impl<V> From<crate::Result<V>> for WaitResult<V> {
    fn from(r: crate::Result<V>) -> Self {
        match r {
            Ok(Some(v)) => Self::Value(v),
            Ok(None) => Self::Stopped,
            Err(e) => Self::Error(e),
        }
    }
}

pub struct AsyncValue<T> {
    value: Mutex<Option<T>>,
    cv: Condvar,
}

impl<T> AsyncValue<T> {
    pub fn new() -> Self {
        Self {
            value: Mutex::new(None),
            cv: Condvar::new(),
        }
    }

    pub fn take(&self) -> T {
        let lock = self.value.lock().unwrap();
        let mut lock = self.cv.wait_while(lock, |x| x.is_none()).unwrap();

        if let Some(x) = lock.take() {
            x
        } else {
            unsafe { unreachable_unchecked() }
        }
    }

    pub fn set(&self, value: T) {
        {
            let mut lock = self.value.lock().unwrap();
            *lock = Some(value);
        }
        self.cv.notify_one();
    }
}

pub struct State<S: Sender> {
    value: AsyncValue<WaitResult<S::Output>>,
}

unsafe impl<S: Sender> Sync for State<S> {}

impl<S: Sender> State<S> {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            value: AsyncValue::new(),
        })
    }

    pub(crate) fn wait_result(self: Arc<Self>) -> WaitResult<S::Output> {
        self.value.take()
    }

    fn set_result(self: Arc<Self>, result: WaitResult<S::Output>) {
        self.value.set(result);
    }
}

pub struct SyncWaitReceiver<S: Sender> {
    state: Arc<State<S>>,
}

impl<S: Sender> SyncWaitReceiver<S> {
    fn new(state: Arc<State<S>>) -> Self {
        Self { state }
    }
}

impl<S: Sender> Receiver for SyncWaitReceiver<S> {
    type Input = S::Output;

    fn set_value(self, value: Self::Input) {
        self.state.set_result(WaitResult::Value(value));
    }

    fn set_error(self, error: crate::Error) {
        self.state.set_result(WaitResult::Error(error));
    }

    fn set_stopped(self) {
        self.state.set_result(WaitResult::Stopped);
    }
}

pub fn sync_wait<S: 'static + Sender>(sender: S) -> WaitResult<S::Output> {
    let state: Arc<State<S>> = State::new();
    let op = sender.connect(SyncWaitReceiver::new(Arc::clone(&state)));
    op.start();
    state.wait_result()
}
