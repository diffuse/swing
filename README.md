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

# Quick start
Create a `Config` to configure your logger, then create and initialize a `DiscoLogger`:
```rust
use disco::{DiscoLogger, Config};
use log::LevelFilter;

fn main() {
    // setup logger
    let config = Config {
        level: LevelFilter::Trace,
        ...Default::default()
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

# Logger config options
A `DiscoLogger` config can be created with:
```rust
let config = Config {
    level: LevelFilter::Trace,
    ...Default::default()
};
```

Below is a breakdown of available config options and their effects (see below subsections for full
explanations of options and behaviors):

Option | Description | Example Usage
--- | --- | ---
`level` | Only logs at or above this severity will be logged | `level: LevelFilter::Debug`
`record_format` | Sets the method used to structure a log line/record | `record_format: RecordFormat::Json`

## level
The `LevelFilter` enum used in `Config` is taken directly from [the log crate](https://docs.rs/log/latest/log/enum.LevelFilter.html).  It defines the following variants:
- `Off`
- `Trace`
- `Debug`
- `Info`
- `Warn`
- `Error`

## record_format
Each call to the [log](https://docs.rs/log/latest/log/) crate macros (`trace!`, `info!`, etc...) generates a log [record](https://docs.rs/log/latest/log/struct.Record.html).  These records are then formatted by this crate using one of the methods in the `RecordFormat` enum:
- `Json`
- `Simple`
- `Custom`

### JSON format
This is the default record format and will generate log lines that look like this:
```json
{"time":"2022-05-30T21:26:06.221369768+00:00","level":"TRACE","target":"main","message":"foo"}
{"time":"2022-05-30T21:26:06.221467684+00:00","level":"DEBUG","target":"main","message":"bar"}
{"time":"2022-05-30T21:26:06.221535118+00:00","level":"INFO","target":"main","message":"baz"}
{"time":"2022-05-30T21:26:06.221589773+00:00","level":"WARN","target":"main","message":"spam"}
{"time":"2022-05-30T21:26:06.221661633+00:00","level":"ERROR","target":"main","message":"eggs"}
```

### Simple format
This format generates log lines that look like this:
```text
2022-05-30T21:25:39.507718423+00:00 [main] TRACE - foo
2022-05-30T21:25:39.507775483+00:00 [main] DEBUG - bar
2022-05-30T21:25:39.507790185+00:00 [main] INFO - baz
2022-05-30T21:25:39.507802207+00:00 [main] WARN - spam
2022-05-30T21:25:39.507830979+00:00 [main] ERROR - eggs
```

### Custom format
If you don't like any of the above formats, you can handle formatting log records directly, using the `Custom` format:
```rust
let record_format = RecordFormat::Custom(Box::new(|r| format!("{} {}", r.level(), r.args())));

let config = Config {
    level: LevelFilter::Trace,
    record_format,
    ..Default::default()
};
```

The above example format will generate log lines that look like this:
```text
TRACE foo
DEBUG bar
INFO baz
WARN spam
ERROR eggs
```

See [the log crate "Record" struct](https://docs.rs/log/latest/log/struct.Record.html) for available record fields/methods to use within the custom format closure.

# Level handling
Logs at levels `trace`, `debug`, and `info` are all written to stdout, while those at `warn` and `error` levels are logged to stderr.

# Example
An example is included at `examples/main.rs` that logs some test messages at different levels.

```shell
$ cargo run
{"time":"2022-05-28T23:08:18.420138779+00:00","level":"TRACE","target":"main","message":"foo"}
{"time":"2022-05-28T23:08:18.420226306+00:00","level":"DEBUG","target":"main","message":"bar"}
{"time":"2022-05-28T23:08:18.420267953+00:00","level":"INFO","target":"main","message":"baz"}
{"time":"2022-05-28T23:08:18.420306418+00:00","level":"WARN","target":"main","message":"spam"}
{"time":"2022-05-28T23:08:18.420361151+00:00","level":"ERROR","target":"main","message":"eggs"}
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
