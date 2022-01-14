use crate::traits::{OperationState, Receiver, Sender};

pub struct Just<T>(pub T);

pub struct JustOperationState<T, R> {
    value: T,
    receiver: R,
}

impl<T, R> JustOperationState<T, R> {
    pub fn new(value: T, receiver: R) -> Self {
        Self { value, receiver }
    }
}

impl<T, R> OperationState for JustOperationState<T, R>
where
    R: Receiver<Input = T>,
{
    fn start(self) {
        self.receiver.set_value(self.value);
    }
}

impl<T> Sender for Just<T>
where
    T: Send + 'static,
{
    type Output = T;
    type ConnectReturnType<R>
    where
        R: Receiver<Input = Self::Output> + Send + 'static,
    = JustOperationState<T, R>;

    fn connect<R>(self, receiver: R) -> JustOperationState<T, R>
    where
        R: Receiver<Input = Self::Output> + Send + 'static,
    {
        JustOperationState::new(self.0, receiver)
    }
}
