//! Behavioral AVR analog-to-digital converter model.
//!
//! The converter does not model analog voltages. Each channel stores a
//! 10-bit sample, and a completed conversion copies the selected channel
//! value into the data registers. When a channel has no external input,
//! the value written through `write_data` is used as the sample.
//!
//! A conversion takes 13 ADC clocks. Prescaler selections 0 through 7
//! select dividers 2, 2, 4, 8, 16, 32, 64, and 128. The reference
//! selection is stored but does not scale samples, which are raw codes.

const MAX_CHANNELS: usize = 16;
const RESULT_MASK: u16 = 0x03FF;
const HIGH_MASK: u8 = 0x03;
const REFERENCE_MASK: u8 = 0xC0;
const CHANNEL_MASK: u8 = 0x0F;
const ENABLE: u8 = 1 << 7;
const START: u8 = 1 << 6;
const FLAG: u8 = 1 << 4;
const INTERRUPT_ENABLE: u8 = 1 << 3;
const PRESCALER_MASK: u8 = 0x07;
const CONTROL_B_MASK: u8 = ENABLE | START | FLAG | INTERRUPT_ENABLE | PRESCALER_MASK;
const CONVERSION_CLOCKS: u32 = 13;
const PRESCALER_DIVIDERS: [u32; 8] = [2, 2, 4, 8, 16, 32, 64, 128];

/// Behavioral model of the AVR 10-bit ADC.
pub struct Adc {
    admux: u8,
    adcsra: u8,
    data: u16,
    next_result: u16,
    inputs: [Option<u16>; MAX_CHANNELS],
    elapsed: u32,
    duration: u32,
    busy: bool,
}

impl Adc {
    /// Builds an ADC in its reset state with no channel inputs configured.
    #[must_use]
    pub fn new() -> Self {
        Self {
            admux: 0,
            adcsra: 0,
            data: 0,
            next_result: 0,
            inputs: [None; MAX_CHANNELS],
            elapsed: 0,
            duration: 0,
            busy: false,
        }
    }

    /// Stores ADMUX, keeping the reference and channel select bits.
    pub fn write_control_a(&mut self, value: u8) {
        self.admux = value & (REFERENCE_MASK | CHANNEL_MASK);
    }

    /// Stores ADCSRA and starts a conversion when the start bit is set.
    pub fn write_control_b(&mut self, value: u8) {
        self.adcsra = value & CONTROL_B_MASK;
        if value & START == 0 {
            return;
        }
        self.start();
    }

    /// Stores the 10-bit result used when the selected channel has no input.
    pub fn write_data(&mut self, low: u8, high: u8) {
        let value = u16::from(low) | (u16::from(high & HIGH_MASK) << 8);
        self.next_result = value;
    }

    /// Stores a 10-bit external input for one channel.
    pub fn set_input(&mut self, channel: u8, value: u16) {
        let index = usize::from(channel);
        if index >= MAX_CHANNELS {
            return;
        }
        self.inputs[index] = Some(value & RESULT_MASK);
    }

    /// Starts a conversion when the ADC is enabled and idle.
    pub fn start(&mut self) {
        if !self.enabled() {
            self.adcsra &= !START;
            return;
        }
        if self.busy {
            return;
        }
        self.busy = true;
        self.elapsed = 0;
        self.duration = conversion_cycles(self.prescaler());
        self.adcsra |= START;
    }

    /// Advances the conversion by CPU cycles.
    pub fn tick(&mut self, cycles: u32) {
        if !self.busy {
            return;
        }
        if cycles == 0 {
            return;
        }
        self.elapsed = self.elapsed.saturating_add(cycles);
        if self.elapsed < self.duration {
            return;
        }
        self.finish_conversion();
    }

    /// Returns the low byte of the last conversion result.
    #[must_use]
    pub fn read_data_low(&self) -> u8 {
        self.data as u8
    }

    /// Returns the high byte of the last conversion result.
    #[must_use]
    pub fn read_data_high(&self) -> u8 {
        (self.data >> 8) as u8
    }

    /// Returns the conversion complete flag (ADIF).
    #[must_use]
    pub fn flag(&self) -> bool {
        self.adcsra & FLAG != 0
    }

    /// Takes and clears the conversion complete flag (ADIF).
    pub fn take_flag(&mut self) -> bool {
        if !self.flag() {
            return false;
        }
        self.adcsra &= !FLAG;
        true
    }

    /// Returns true while a conversion is in progress.
    #[must_use]
    pub fn busy(&self) -> bool {
        self.busy
    }

    /// Returns the selected ADC channel.
    #[must_use]
    pub fn channel(&self) -> u8 {
        self.admux & CHANNEL_MASK
    }

    fn enabled(&self) -> bool {
        self.adcsra & ENABLE != 0
    }

    fn prescaler(&self) -> u8 {
        self.adcsra & PRESCALER_MASK
    }

    fn conversion_value(&self) -> u16 {
        let channel = usize::from(self.channel());
        self.inputs[channel].unwrap_or(self.next_result) & RESULT_MASK
    }

    fn finish_conversion(&mut self) {
        self.data = self.conversion_value();
        self.busy = false;
        self.elapsed = 0;
        self.adcsra &= !START;
        self.adcsra |= FLAG;
    }
}

impl Default for Adc {
    fn default() -> Self {
        Self::new()
    }
}

fn conversion_cycles(select: u8) -> u32 {
    let divider = PRESCALER_DIVIDERS[usize::from(select & PRESCALER_MASK)];
    CONVERSION_CLOCKS.saturating_mul(divider)
}

#[cfg(test)]
mod tests {
    use super::{Adc, ENABLE, MAX_CHANNELS};

    fn enabled_adc(prescaler: u8) -> Adc {
        let mut adc = Adc::new();
        adc.write_control_b(ENABLE | prescaler);
        adc
    }

    fn convert_once(adc: &mut Adc) {
        adc.start();
        adc.tick(26);
    }

    #[test]
    fn disabled_adc_ignores_start() {
        let mut adc = Adc::new();
        adc.write_data(0x55, 0x01);
        adc.start();
        adc.tick(1_000);
        assert!(!adc.busy());
        assert!(!adc.flag());
        assert_eq!(adc.read_data_low(), 0);
    }

    #[test]
    fn enabled_adc_converts_after_prescaled_ticks() {
        let mut adc = enabled_adc(0);
        adc.start();
        assert!(adc.busy());
        adc.tick(25);
        assert!(adc.busy());
        assert!(!adc.flag());
        adc.tick(1);
        assert!(!adc.busy());
        assert!(adc.flag());
    }

    #[test]
    fn conversion_sets_and_take_clears_flag() {
        let mut adc = enabled_adc(0);
        adc.write_data(0x34, 0x02);
        convert_once(&mut adc);
        assert!(adc.flag());
        assert!(adc.take_flag());
        assert!(!adc.flag());
        assert!(!adc.take_flag());
    }

    #[test]
    fn conversion_stores_ten_bit_result() {
        let mut adc = enabled_adc(0);
        adc.write_data(0xCD, 0x01);
        convert_once(&mut adc);
        assert_eq!(adc.read_data_low(), 0xCD);
        assert_eq!(adc.read_data_high(), 0x01);
    }

    #[test]
    fn channel_selects_external_input() {
        let mut adc = enabled_adc(0);
        adc.set_input(2, 0x0155);
        adc.set_input(3, 0x02AA);
        adc.write_control_a(3);
        assert_eq!(adc.channel(), 3);
        convert_once(&mut adc);
        assert_eq!(adc.read_data_low(), 0xAA);
        assert_eq!(adc.read_data_high(), 0x02);
    }

    #[test]
    fn external_input_overrides_programmed_result() {
        let mut adc = enabled_adc(0);
        adc.write_data(0x00, 0x00);
        adc.set_input(0, 0x0201);
        convert_once(&mut adc);
        assert_eq!(adc.read_data_low(), 0x01);
        assert_eq!(adc.read_data_high(), 0x02);
    }

    #[test]
    fn prescaler_extends_conversion_time() {
        let mut adc = enabled_adc(2);
        adc.start();
        adc.tick(51);
        assert!(adc.busy());
        adc.tick(1);
        assert!(!adc.busy());
    }

    #[test]
    fn write_data_masks_to_ten_bits() {
        let mut adc = enabled_adc(0);
        adc.write_data(0xFF, 0xFF);
        convert_once(&mut adc);
        assert_eq!(adc.read_data_low(), 0xFF);
        assert_eq!(adc.read_data_high(), 0x03);
    }

    #[test]
    fn channel_is_masked_to_four_bits() {
        let mut adc = Adc::new();
        adc.write_control_a(0xFF);
        assert_eq!(adc.channel(), 0x0F);
    }

    #[test]
    fn out_of_range_input_channel_is_ignored() {
        let mut adc = enabled_adc(0);
        adc.set_input(MAX_CHANNELS as u8, 0x03FF);
        convert_once(&mut adc);
        assert_eq!(adc.read_data_low(), 0);
    }

    #[test]
    fn start_bit_in_control_b_starts_conversion() {
        let mut adc = Adc::new();
        adc.write_control_b(ENABLE | super::START);
        assert!(adc.busy());
        adc.tick(26);
        assert!(adc.flag());
    }

    #[test]
    fn default_matches_new() {
        let adc = Adc::default();
        assert!(!adc.busy());
        assert!(!adc.flag());
        assert_eq!(adc.channel(), 0);
        assert_eq!(adc.read_data_low(), 0);
        assert_eq!(adc.read_data_high(), 0);
    }

    #[test]
    fn source_is_ascii() {
        assert!(include_str!("adc.rs").is_ascii());
    }
}
