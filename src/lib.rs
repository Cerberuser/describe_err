use thiserror::Error;
use std::error;

#[derive(Debug, Error)]
#[error("{description}: {inner}")]
pub struct Described<E: std::error::Error + 'static> {
    description: String,
    #[source]
    inner: E,
}

pub fn describe<E: error::Error>(description: impl Into<String>) -> impl FnOnce(E) -> Described<E> {
    let description = description.into();
    |inner| Described { description, inner }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn simple_error() {
        let err: Result<(), _> = Err(std::io::Error::new(std::io::ErrorKind::Other, "Inner error")).map_err(describe("Produced in test"));
        let err = err.unwrap_err();
        assert_eq!(err.to_string(), "Produced in test: Inner error");
    }

}