#![feature(generic_associated_types)]

mod factories;
mod traits;
mod consumers;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<Option<T>, crate::Error>;

#[cfg(test)]
mod tests {
    use crate::factories::just;
    use crate::consumers::sync_wait;

    #[test]
    fn test_just() {
        let res = sync_wait(just(4));
        assert_eq!(res.unwrap(), 4);
    }
}
