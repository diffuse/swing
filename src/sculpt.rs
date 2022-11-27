use log::Record;
use serde_json::json;
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

/// Record formatting mode that determines how log records are structured
pub enum RecordFormat {
    /// JSON format
    Json,
    /// simple log format `<timestamp> [<target>] - <message>`
    Simple,
    /// custom record formatter provided by client code
    Custom(Box<dyn Sync + Send + Fn(&Record) -> String>),
}

/// Sculpt/create structurally formatted string logs from raw log records
pub struct LogSculptor {
    /// record formatting mode (determines how log records are structurally formatted)
    pub record_format: RecordFormat,
}

impl LogSculptor {
    /// Create a new LogSculptor using the provided record format
    ///
    /// # Arguments
    ///
    /// * `record_format` - the structural format to use when sculpting records
    pub fn new(record_format: RecordFormat) -> LogSculptor {
        LogSculptor { record_format }
    }

    /// Convert a log record into a formatted string, based on the current logger configuration
    ///
    /// # Arguments
    ///
    /// * `record` - the log record to format
    pub fn sculpt(&self, record: &Record) -> String {
        let now = OffsetDateTime::now_utc()
            .format(&Iso8601::DEFAULT)
            .expect("Failed to format time as ISO 8601");

        match &self.record_format {
            RecordFormat::Json => json!({
                "time": now,
                "level": record.level(),
                "target": record.target(),
                "message": record.args(),
            })
            .to_string(),
            RecordFormat::Simple => {
                format!(
                    "{} [{}] {} - {}",
                    now,
                    record.target(),
                    record.level(),
                    record.args()
                )
            }
            RecordFormat::Custom(f) => f(record),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::Level;

    #[test]
    fn sculpt_presets_return_non_empty() {
        for fmt in vec![RecordFormat::Json, RecordFormat::Simple] {
            let sculptor = LogSculptor::new(fmt);

            // create normal test record
            let rec = Record::builder()
                .args(format_args!("foo"))
                .level(Level::Info)
                .target("test")
                .build();

            assert!(!sculptor.sculpt(&rec).is_empty());

            // create record with empty args and target
            let rec = Record::builder()
                .args(format_args!(""))
                .level(Level::Info)
                .target("")
                .build();

            // record should still give non-empty log lines
            assert!(!sculptor.sculpt(&rec).is_empty());
        }
    }

    #[test]
    fn sculpt_custom_formats_correctly() {
        let test_cases = vec![
            (RecordFormat::Custom(Box::new(|_| "".to_string())), ""),
            (
                RecordFormat::Custom(Box::new(|r| format!("{} {}", r.level(), r.args()))),
                "INFO foo",
            ),
            (
                RecordFormat::Custom(Box::new(|r| {
                    format!("{} [{}] {}", r.level(), r.target(), r.args())
                })),
                "INFO [test] foo",
            ),
        ];

        let rec = Record::builder()
            .args(format_args!("foo"))
            .level(Level::Info)
            .target("test")
            .build();

        for (fmt, expected) in test_cases {
            let sculptor = LogSculptor::new(fmt);
            assert_eq!(sculptor.sculpt(&rec), expected);
        }
    }
}
