<img src="images/disco.gif" alt="disco logo" width="25%"/>

# disco
Log like it's 1978 with this logging implementation for the [log](https://crates.io/crates/log) crate.

# Installation
Add the following to `Cargo.toml`:
```toml
[dependencies]
disco = "0.1"
log = "0.4"
```

# Usage
Create a `LoggerConfig` to configure your logger, then create and initialize a `DiscoLogger`:
```rust
use disco::{DiscoLogger, LoggerConfig};
use log::LevelFilter;

fn main() {
    // setup logger
    let config = LoggerConfig {
        level: LevelFilter::Trace,
    };
    DiscoLogger::new(config).init().unwrap();

    // log away!
    log::trace!("foo");
    log::debug!("bar");
    log::info!("baz");
    log::warn!("spam");
    log::error!("eggs");
}
```

# Level handling
Logs at levels `trace`, `debug`, and `info` are all written to stdout, while those at `warn` and `error` levels are logged to stderr.

# Example output
```shell
$ ./example
{"time":"2022-02-11T02:06:08.172675662+00:00","level":"TRACE","target":"example","message":"foo"}
{"time":"2022-02-11T02:06:08.172740488+00:00","level":"DEBUG","target":"example","message":"bar"}
{"time":"2022-02-11T02:06:08.172794482+00:00","level":"INFO","target":"example","message":"baz"}
{"time":"2022-02-11T02:06:08.172836567+00:00","level":"WARN","target":"example","message":"spam"}
{"time":"2022-02-11T02:06:08.172923387+00:00","level":"ERROR","target":"example","message":"eggs"}
```

# Stream redirection
Since this logger writes to both stdout and stderr, you must redirect both streams to capture all output.

## Redirect all logs to file
```shell
$ ./example &> foo.log
```

## Redirect all logs to file, while watching output
Write all log data to `foo.log` and stdout:
```shell
$ ./example 2>&1 | tee foo.log
```

You can add `jq` for pretty printing:
```shell
$ ./example 2>&1 | tee foo.log | jq
```