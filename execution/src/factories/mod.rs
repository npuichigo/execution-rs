pub(crate) mod just_sender;

pub fn just<T>(value: T) -> just_sender::Just<T> {
    just_sender::Just(value)
}
