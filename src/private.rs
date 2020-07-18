use crate::anyhow::Error;
pub trait Sealed {}

impl<T, E> Sealed for std::result::Result<T, E> where E: Into<Error> {}
