//! CSV and VCD board trace output.

use sim_core::SimError;

/// Returns the CSV header.
#[must_use]
pub const fn csv_header() -> &'static str {
    "cycle,pb5,led"
}

/// Formats one CSV row.
#[must_use]
pub fn csv_line(cycle: u64, pb5: bool, led: bool) -> String {
    format!("{cycle},{},{}\n", u8::from(pb5), u8::from(led))
}

/// Generic CSV writer with fixed columns.
pub struct CsvWriter {
    text: String,
    columns: usize,
}

impl CsvWriter {
    /// Creates a writer with a header row.
    #[must_use]
    pub fn new(columns: &[&str]) -> Self {
        Self {
            text: format!("{}\n", columns.join(",")),
            columns: columns.len(),
        }
    }

    /// Records one row.
    pub fn record(&mut self, values: &[f64]) -> Result<(), SimError> {
        if values.len() != self.columns {
            return Err(SimError::DimensionMismatch {
                expected: self.columns,
                actual: values.len(),
            });
        }
        let fields: Vec<String> = values.iter().map(|value| value.to_string()).collect();
        self.text.push_str(&fields.join(","));
        self.text.push('\n');
        Ok(())
    }

    /// Returns the CSV text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Minimal VCD writer for PB5 and LED.
pub struct VcdWriter {
    text: String,
    last_pb5: bool,
    last_led: bool,
}

impl VcdWriter {
    /// Creates a VCD writer with header.
    #[must_use]
    pub fn new() -> Self {
        let mut text = String::from("$timescale 1ns $end\n");
        text.push_str("$var wire 1 ! pb5 $end\n");
        text.push_str("$var wire 1 \" led $end\n");
        text.push_str("$enddefinitions $end\n");
        Self {
            text,
            last_pb5: false,
            last_led: false,
        }
    }

    /// Records signal values at a cycle.
    pub fn record(&mut self, cycle: u64, pb5: bool, led: bool) {
        if pb5 != self.last_pb5 {
            self.text
                .push_str(&format!("#{cycle}\n{}!\n", u8::from(pb5)));
            self.last_pb5 = pb5;
        }
        if led != self.last_led {
            self.text
                .push_str(&format!("#{cycle}\n{}\"\n", u8::from(led)));
            self.last_led = led;
        }
    }

    /// Returns the VCD text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl Default for VcdWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// VCD writer for real-valued signals.
pub struct AnalogVcdWriter {
    text: String,
    last: Vec<f64>,
}

impl AnalogVcdWriter {
    /// Creates a writer for named real signals.
    #[must_use]
    pub fn new(signals: &[&str]) -> Self {
        let mut text = String::from("$timescale 1ns $end\n");
        for (index, name) in signals.iter().enumerate() {
            text.push_str(&format!("$var real 64 v{index} {name} $end\n"));
        }
        text.push_str("$enddefinitions $end\n");
        Self {
            text,
            last: vec![f64::NAN; signals.len()],
        }
    }

    /// Records real values at a time in nanoseconds.
    pub fn record(&mut self, time_ns: u64, values: &[f64]) -> Result<(), SimError> {
        if values.len() != self.last.len() {
            return Err(SimError::DimensionMismatch {
                expected: self.last.len(),
                actual: values.len(),
            });
        }
        for (index, value) in values.iter().enumerate() {
            if self.last[index].to_bits() != value.to_bits() {
                self.text
                    .push_str(&format!("#{time_ns}\nr{value} v{index}\n"));
                self.last[index] = *value;
            }
        }
        Ok(())
    }

    /// Returns the VCD text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

#[cfg(test)]
mod tests {
    use super::{AnalogVcdWriter, CsvWriter, VcdWriter, csv_header, csv_line};

    #[test]
    fn csv_formats_header_and_row() {
        assert_eq!(csv_header(), "cycle,pb5,led");
        assert_eq!(csv_line(5, true, false), "5,1,0\n");
    }

    #[test]
    fn csv_writer_records_columns() {
        let mut csv = CsvWriter::new(&["time", "voltage"]);
        csv.record(&[0.0, 1.5]).expect("row");
        assert_eq!(csv.text(), "time,voltage\n0,1.5\n");
        assert!(csv.record(&[1.0]).is_err());
    }

    #[test]
    fn analog_vcd_records_real_values() {
        let mut vcd = AnalogVcdWriter::new(&["vout"]);
        vcd.record(10, &[1.5]).expect("record");
        vcd.record(10, &[1.5]).expect("record");
        vcd.record(20, &[2.5]).expect("record");
        assert!(vcd.text().contains("$var real 64 v0 vout $end"));
        assert!(vcd.text().contains("#10\nr1.5 v0\n"));
        assert!(vcd.text().contains("#20\nr2.5 v0\n"));
        assert!(vcd.record(30, &[1.0, 2.0]).is_err());
    }

    #[test]
    fn vcd_records_changes() {
        let mut vcd = VcdWriter::new();
        vcd.record(10, true, true);
        vcd.record(10, true, true);
        vcd.record(20, false, true);
        let text = vcd.text();
        assert!(text.contains("$var wire 1 ! pb5 $end"));
        assert!(text.contains("#10\n1!\n"));
        assert!(text.contains("#20\n0!\n"));
    }
}
