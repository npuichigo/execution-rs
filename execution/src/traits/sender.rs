use crate::traits::OperationState;
use crate::traits::Receiver;

pub trait Sender {
    type Output: Send + 'static;

    type ConnectReturnType<R>: OperationState
    where
        R: Receiver<Input = Self::Output> + Send + 'static;

    fn connect<R>(self, receiver: R) -> Self::ConnectReturnType<R>
    where
        R: Receiver<Input = Self::Output> + Send + 'static;
}
