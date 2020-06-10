## describe_err

Wonder where this error comes from?

Trying to find out which of the multiple IO errors ruins your day?

Now you can add a short description to every error and be sure to know what's going on. And if you don't want to invent them yourself, well, you can just generate them from the code!

### Examples

Imagine that you want to create file on the given path and write here a given string. Let's forget for a moment that `std::fs::write` exists and do it ourselves:
```rust
fn create_and_write(path: &Path, content: &str) -> Result<(), io::Error> {
    let file = File::create(path)?;
    write!(file, "{}", content)?;
    file.sync_all()
}
```
Here are three distinct sources of error, and it might not always be obvious which of them is the real one in particular case. That's how it is handled with `describe_err`:
```rust
use describe_err::{describing, Described};

fn create_and_write(path: &Path, content: &str) -> Result<(), Described<io::Error>> {
    let file = describing!(File::create(path))?;
    write!(file, "{}", content).map_err(describe("Cannot write to file"))?;
    describing!(file.sync_all())
}
```

Here you can see two ways to use the library:

- By explicitly providing the description with `describe`. This function returns the closure, which maps an incoming error to `Described` instance.
- By wrapping the `Result`-producing operation in `describing!` macro, which will describe the error with the stringified content.

### What can we do with this wrapper?

Just everything you'd do with any other error.

`Described` implements `Display` by concatenating the provided description and the `Display` output of original error, separated by colon. It also implements `Error`, with `Error::cause` pointing on the original error. So, you can easily combine it with, for example, [`thiserror`](https://crates.io/crate/thiserror) - which, by the way, is what powers the `Described` itself.

## License

MIT (c) Cerberuser