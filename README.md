[![check](https://github.com/Defelo/tokio-shield/actions/workflows/check.yml/badge.svg)](https://github.com/Defelo/tokio-shield/actions/workflows/check.yml)
[![test](https://github.com/Defelo/tokio-shield/actions/workflows/test.yml/badge.svg)](https://github.com/Defelo/tokio-shield/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/Defelo/tokio-shield/branch/develop/graph/badge.svg?token=D6G8P3ZJD6)](https://codecov.io/gh/Defelo/tokio-shield)
![Version](https://img.shields.io/github/v/tag/Defelo/tokio-shield?include_prereleases&label=version)
[![dependency status](https://deps.rs/repo/github/Defelo/tokio-shield/status.svg)](https://deps.rs/repo/github/Defelo/tokio-shield)

# tokio-shield
Prevent futures in Rust from being aborted by wrapping them in tasks.

## Example
```rust
use std::time::Duration;
use tokio::{sync::oneshot, time::sleep};
use tokio_shield::Shield;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = oneshot::channel();

    // Create and shield a future that waits for 10ms and then returns a value
    // via a oneshot channel.
    let future = async {
        sleep(Duration::from_millis(10)).await;
        tx.send(42).unwrap();
    }.shield();

    // Spawn a task to run this future, but cancel it after 5ms.
    let task = tokio::spawn(future);
    sleep(Duration::from_millis(5)).await;
    task.abort();
    sleep(Duration::from_millis(5)).await;

    // After 10ms the value can successfully be read from the oneshot channel,
    // because `shield` prevented our future from being canceled.
    assert_eq!(rx.try_recv().unwrap(), 42);
}
```
