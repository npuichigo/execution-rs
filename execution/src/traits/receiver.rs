pub trait Receiver {
    type Input;

    fn set_value(self, value: Self::Input);
    fn set_error(self, error: crate::Error);
    fn set_stopped(self);
}
