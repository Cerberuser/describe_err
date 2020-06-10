//! This library provides an error wrapper which adds a description to its specific instance.
//! 
//! ### Examples
//! 
//! For example, you want to create file on the given path and write here a given string.
//! Let's forget for a moment that [`std::fs::write`] exists and do it ourselves:
//! ```
//! # use std::{io, fs::File, path::Path};
//! use std::io::Write;
//! fn create_and_write(path: &Path, content: &str) -> Result<(), io::Error> {
//!     let mut file = File::create(path)?;
//!     write!(file, "{}", content)?;
//!     file.sync_all()
//! }
//! ```
//! Here are three distinct sources of error, and it might not always be obvious
//! which of them is the real one in particular case. That's how it is handled with `describe_err`:
//! ```
//! # use std::{io, fs::File, path::Path};
//! use std::io::Write;
//! use describe_err::{describing, describe, Described};
//! 
//! fn create_and_write(path: &Path, content: &str) -> Result<(), Described<io::Error>> {
//!     let mut file = describing!(File::create(path))?;
//!     write!(file, "{}", content).map_err(describe("Cannot write to file"))?;
//!     describing!(file.sync_all())
//! }
//! ```
//! 
//! Here you can see two ways to use the library:
//! 
//! - By explicitly providing the description with [`describe`].
//!     This function returns the closure, which maps an incoming error to `Described` instance.
//! - By wrapping the `Result`-producing operation in [`describing!`] macro,
//!     which will describe the error with the stringified content.
//! 
//! And here's how will be used the generated output:
//! ```
//! # use std::{io, fs::File, path::{Path, PathBuf}};
//! # use std::io::Write;
//! # use describe_err::{describing, describe, Described};
//! # 
//! # fn create_and_write(path: &Path, content: &str) -> Result<(), Described<io::Error>> {
//! #     let mut file = describing!(File::create(path))?;
//! #     write!(file, "{}", content).map_err(describe("Cannot write to file"))?;
//! #     describing!(file.sync_all())
//! # }
//! fn main() {
//!     let path = PathBuf::from("/tmp/nonexistent/path");
//!     let res = create_and_write(&path, "arbitrary content");
//!     let err = res.unwrap_err();
//!     assert_eq!(err.to_string(), "File::create(path): No such file or directory (os error 2)");
//! }
//! ```
//! As you can see, the command which produced an error is right here, in the error itself.

use thiserror::Error;
use std::error;

/// An error wrapper with description.
/// 
/// This struct can hold every error, with the only restriction that this error
/// must be `'static` to support downcasting through [`source`][std::error::Error::source].
/// 
/// When converting this wrapper to string with `Display`, it will render colon-separated
/// pair of description and original error:
/// ```
/// use describe_err::{Described, describing};
/// fn fmt<E: std::error::Error + 'static>(err: &Described<E>) -> String {
///     format!("{}: {}", err.description(), err.original())
/// }
/// 
/// fn main() {
///     // Let's create a simple error with auto-generated description...
///     let res: Result<u32, _> = describing!("Not a number".parse());
///     // ...then unwrap it...
///     let err = res.unwrap_err();
///     // and see that the formatting is indeed the same:
///     assert_eq!(fmt(&err), format!("{}", err));
/// }
/// ```
#[derive(Debug, Error)]
#[error("{description}: {original}")]
pub struct Described<E: error::Error + 'static> {
    description: String,
    #[source]
    original: E,
}

impl<E: error::Error + 'static> Described<E> {
    /// Directly retrieves an error description.
    pub fn description(&self) -> &str {
        &self.description
    }
    /// Directly retrieves an original error.
    /// 
    /// This method is different from [`source`][std::error::Error::source],
    /// since it is generic and is known to return exactly the wrapped type,
    /// not a boxed trait object. This way you won't need any downcasting.
    pub fn original(&self) -> &E {
        &self.original
    }
}

/// Wrap an error with description.
/// 
/// This method generates a closure to be passed into `map_err`:
/// ```
/// use describe_err::describe;
/// let description = "Parsing a not-a-number to number";
/// let err = "Not a number".parse::<u64>().map_err(describe(description)).unwrap_err();
/// assert_eq!(err.description(), description);
/// ```
pub fn describe<E: error::Error>(description: impl Into<String>) -> impl FnOnce(E) -> Described<E> {
    let description = description.into();
    |original| Described { description, original }
}

/// Wrap an error with an auto-generated description.
/// 
/// This macro is essentially a wrapper around [`describe`]. It expands to the following:
/// ```
/// # use describe_err::describe;
/// # let result_expression = "123".parse::<u64>();
/// // let res = describing!(result_expression);
/// let res = result_expression.map_err(describe("result_expression"));
/// ```
/// The returned `Result` can be pattern-matched or propagated as usual.
#[macro_export]
macro_rules! describing {
    ($expr:expr) => {{
        let expr: Result<_, _> = $expr;
        expr.map_err($crate::describe(stringify!($expr)))
    }};
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io;

    #[test]
    fn simple_error() {
        let err: Result<(), _> = Err(io::Error::new(io::ErrorKind::Other, "Inner error")).map_err(describe("Produced in test"));
        let err = err.unwrap_err();
        assert_eq!(err.to_string(), "Produced in test: Inner error");
    }

    fn returns_err() -> Result<(), io::Error> {
        Err(io::Error::new(io::ErrorKind::Other, "Inner error"))
    }

    #[test]
    fn macro_err() {
        let err = describing!(returns_err()).unwrap_err();
        assert_eq!(err.to_string(), "returns_err(): Inner error");
    }

}