//! Builds a three-inverter ring oscillator cell.

use sim_cells::ring_oscillator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cell = ring_oscillator()?;
    println!("output port: {:?}", cell.port("y"));
    println!("vdd port: {:?}", cell.vdd());
    let powered = cell.powered(5.0)?;
    println!("powered circuit nodes: {}", powered.nodes().len());
    Ok(())
}
