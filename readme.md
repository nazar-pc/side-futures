## Side futures

[![Build Status](https://img.shields.io/travis/com/nazar-pc/side-futures/master?style=flat-square)](https://travis-ci.org/nazar-pc/side-futures)
[![Crates.io](https://img.shields.io/crates/v/side-futures?style=flat-square)](https://crates.io/crates/side-futures)
[![Docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/side-futures)
[![License](https://img.shields.io/github/license/nazar-pc/side-futures?style=flat-square)](https://github.com/nazar-pc/side-futures)

This crate provides an ability to send future for execution on the runtime that may be in a different thread. Typical use case is heavily threaded application where there are synchronous callbacks, but some asynchronous tasks also need to be executed.

## Example
To get started, add the following to `Cargo.toml`.

```toml
side-futures = "0.1.0"
```

Typical usage with Tokio runtime:
```rust
use tokio::task;

#[tokio::main]
async fn main() {
    let (sender, receiver) = side_futures::create();
    task::spawn(receiver.run_receiver(task::spawn));
    sender
        .send_future(async {
            // Do stuff
        })
        .unwrap();
}
```

Typical usage with Actix runtime:
```rust
#[actix_rt::main]
async fn main() {
    let (sender, receiver) = side_futures::create();
    actix_rt::spawn(receiver.run_receiver(actix_rt::spawn));
    sender
        .send_future(async {
            // Do stuff
        })
        .unwrap();
}
```

## Contribution
Feel free to create issues and send pull requests, they are highly appreciated!

## License
Zero-Clause BSD

https://opensource.org/licenses/0BSD

https://tldrlegal.com/license/bsd-0-clause-license
