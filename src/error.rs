use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("phone channel is already closed")]
    PhoneClosed,
}
