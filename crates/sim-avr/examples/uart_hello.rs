//! Sends and decodes a UART hello message from the TX line.

use sim_avr::Uart;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut uart = Uart::new();
    uart.write_baud(0, 0);
    uart.write_control(1, 1 << 3);
    let mut decoded = String::new();
    for byte in b"Hello\n" {
        let value = send_byte(&mut uart, *byte)?;
        decoded.push(char::from(value));
    }
    print!("{decoded}");
    Ok(())
}

fn send_byte(uart: &mut Uart, byte: u8) -> Result<u8, Box<dyn std::error::Error>> {
    uart.write_tx(byte);
    let mut value = 0u8;
    for bit in 0..10 {
        uart.tick(8);
        let level = uart.tx_line();
        if (1..=8).contains(&bit) && level {
            value |= 1 << (bit - 1);
        }
        uart.tick(8);
    }
    if value != byte {
        return Err(format!("decoded {value:#04X} instead of {byte:#04X}").into());
    }
    Ok(value)
}
