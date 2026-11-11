//! CSV and VCD board trace output.

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

#[cfg(test)]
mod tests {
    use super::{VcdWriter, csv_header, csv_line};

    #[test]
    fn csv_formats_header_and_row() {
        assert_eq!(csv_header(), "cycle,pb5,led");
        assert_eq!(csv_line(5, true, false), "5,1,0\n");
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
