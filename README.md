<img src="images/disco.gif" alt="disco logo" width="25%"/>

# disco
Log like it's 1978 with this logging implementation for the [log](https://crates.io/crates/log) crate.  Color themes, pluggable formatting, we've got it all!

# Installation
Add the following to `Cargo.toml`:
```toml
[dependencies]
disco = "0.1"
log = "0.4"
```

# Quick start
Create and initialize a `DiscoLogger`, then use the [log](https://crates.io/crates/log) crate macros to log messages:
```rust
use disco::DiscoLogger;

fn main() {
    // setup logger
    DiscoLogger::new().init().unwrap();

    // log away!
    log::trace!("foo");
    log::debug!("bar");
    log::info!("baz");
    log::warn!("spam");
    log::error!("eggs");
}
```
Note that the default `DiscoLogger` created with `::new()` has a log level filter of `info`, so in this example, only `"baz"` `"spam"` and `"eggs"` will be printed to the console.

# Logger config options
For more control, `DiscoLogger`s can be created with a `Config` struct via `DiscoLogger::with_config`:
```rust
let config = Config {
    level: LevelFilter::Trace,
    ...Default::default()
};

DiscoLogger::with_config(config).init().unwrap();
```

The default configuration uses the following settings (explained below):
```rust
Config {
    level: LevelFilter::Info,
    record_format: RecordFormat::Simple,
    color_format: Some(ColorFormat::Solid),
    theme: Box::new(theme::Simple {}),
    use_stderr: true,
}
```

A `DiscoLogger`'s main areas of configurability are:
- color theme
- log format
- `stdout`\/`stderr` splitting

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

### Simple format
This is the default record format and will generate log lines that look like this:
```text
2022-07-31T20:25:31.108560826Z [main] TRACE - foo
2022-07-31T20:25:31.108623041Z [main] DEBUG - bar
2022-07-31T20:25:31.108645580Z [main] INFO - baz
2022-07-31T20:25:31.108667634Z [main] WARN - spam
2022-07-31T20:25:31.108736790Z [main] ERROR - eggs
```

### JSON format
This record format will generate log lines as JSON:
```json
{"time":"2022-07-31T20:28:11.863634602Z","level":"TRACE","target":"main","message":"foo"}
{"time":"2022-07-31T20:28:11.864114090Z","level":"DEBUG","target":"main","message":"bar"}
{"time":"2022-07-31T20:28:11.864201937Z","level":"INFO","target":"main","message":"baz"}
{"time":"2022-07-31T20:28:11.864269093Z","level":"WARN","target":"main","message":"spam"}
{"time":"2022-07-31T20:28:11.864372619Z","level":"ERROR","target":"main","message":"eggs"}
```

### Custom format
If you don't like any of the above formats, you can inject your own custom record formatting by using the `Custom` format:
```rust
let fmt_rec = Box::new(|r: &Record| -> String {
    format!("{} - {}", r.level(), r.args())
});

let config = Config {
    level: LevelFilter::Trace,
    record_format,
    ..Default::default()
};
```

The above example format will generate log lines that look like this:
```text
TRACE - foo
DEBUG - bar
INFO - baz
WARN - spam
ERROR - eggs
```

See [the log crate "Record" struct](https://docs.rs/log/latest/log/struct.Record.html) for available record fields/methods to use within the custom format closure.

For reference, the `Simple` record format can be reproduced with the following `Custom` record format:
```rust
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

// --snip--

let fmt_rec = Box::new(|r: &Record| -> String {
    let now = OffsetDateTime::now_utc()
        .format(&Iso8601::DEFAULT)
        .expect("Failed to format time as ISO 8601");

    format!("{} [{}] {} - {}", now, r.target(), r.level(), r.args())
});
```

### stderr handling
The `use_stderr` `Config` setting changes how log records are split between `stdout` and `stderr`.  When this field is false, all log records will be written to `stdout`.  When this field is true, records at levels `trace`, `debug`, and `info` are written to stdout, while those at `warn` and `error` levels are written to stderr.

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
