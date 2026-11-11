//! `embed-emu` command-line entry point.

mod monitor;

use monitor::{Monitor, parse_number, parse_word};
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(message) = run(&args) {
        eprintln!("error: {message}");
        eprintln!("{}", usage());
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let (command, words) = args.split_first().ok_or_else(usage)?;
    if command == "disasm" {
        return show_disassembly(words);
    }
    if command == "monitor" {
        let mut monitor = Monitor::new(parse_program(words)?)?;
        return repl(&mut monitor);
    }
    Err(usage())
}

fn show_disassembly(words: &[String]) -> Result<(), String> {
    let monitor = Monitor::new(parse_program(words)?)?;
    println!("{}", monitor.disassemble(0, words.len())?);
    Ok(())
}

fn parse_program(words: &[String]) -> Result<Vec<u16>, String> {
    if words.is_empty() {
        return Err("program is empty".to_string());
    }
    words.iter().map(|word| parse_word(word)).collect()
}

fn repl(monitor: &mut Monitor) -> Result<(), String> {
    let stdin = io::stdin();
    loop {
        print!("avr> ");
        io::stdout().flush().map_err(|error| error.to_string())?;
        let mut line = String::new();
        if stdin
            .read_line(&mut line)
            .map_err(|error| error.to_string())?
            == 0
        {
            return Ok(());
        }
        if !execute(monitor, line.trim())? {
            return Ok(());
        }
    }
}

fn execute(monitor: &mut Monitor, line: &str) -> Result<bool, String> {
    let mut parts = line.split_whitespace();
    let Some(command) = parts.next() else {
        return Ok(true);
    };
    let args: Vec<&str> = parts.collect();
    match command {
        "quit" | "q" => return Ok(false),
        "step" => println!("{}", monitor.step()?),
        "run" => println!("{}", monitor.run(number(&args, 0, 100)?)?),
        "regs" => println!("{}", monitor.registers()),
        "sreg" => println!("{}", monitor.sreg()),
        "pc" => println!("0x{:04X} cycles={}", monitor.pc(), monitor.cycles()),
        "disasm" => run_disasm(monitor, &args)?,
        "mem" => run_mem(monitor, &args)?,
        "io" => println!("{}", monitor.io(address(&args, 0)? as u8)),
        _ => println!("unknown command: {command}"),
    }
    Ok(true)
}

fn run_disasm(monitor: &Monitor, args: &[&str]) -> Result<(), String> {
    let address = address(args, 0)?;
    let count = number(args, 1, 1)?;
    println!("{}", monitor.disassemble(address, count)?);
    Ok(())
}

fn run_mem(monitor: &Monitor, args: &[&str]) -> Result<(), String> {
    let address = address(args, 0)?;
    let length = number(args, 1, 16)?;
    println!("{}", monitor.memory(address, length));
    Ok(())
}

fn address(args: &[&str], index: usize) -> Result<u16, String> {
    Ok(number(args, index, 0)? as u16)
}

fn number(args: &[&str], index: usize, default: usize) -> Result<usize, String> {
    args.get(index).map_or(Ok(default), |raw| parse_number(raw))
}

fn usage() -> String {
    "usage: embed-emu monitor <hex words...> | embed-emu disasm <hex words...>".to_string()
}
