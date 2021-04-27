mod parsers;

use parsers::Movie;
use serialport::{SerialPort, ClearBuffer};
use std::io::Write;
use std::time::Duration;

fn main() {
    let mut port = load_port(true).unwrap();
    let mut loaded_movie = Movie::none();
    
    let mut running = true;
    while running {
        let cli_input = read_cli();
        let cli_parts: Vec<&str> = cli_input.split(" ").collect();
        
        port.clear(ClearBuffer::All).unwrap();
        
        loop { match cli_parts[0] {
            "clear" => {
                port.clear(ClearBuffer::All).unwrap();
            },
            
            "load" => {
                if cli_parts.len() < 2 {
                    println!("You must include a path to the movie file!");
                    break;
                }
                loaded_movie = parsers::parse(cli_parts[1]);
                println!("LEN: {}", loaded_movie.inputs.len());
                
                /*let mut prepends = Vec::<u8>::new();
                prepends.push(0x10);
                prepends.push(0x00);
                
                for _ in 0..126 {
                    prepends.push(0x00);
                    prepends.push(0x00);
                }
                
                loaded_movie.prepend(&prepends, true);*/
            },
            
            "port" => { port = load_port(false).unwrap() },
            
            "ping" => { ping(&mut port) },
            
            "program" => { program(&mut port, &mut loaded_movie) },
            
            "everdrive" => { everdrive_start(&mut port) },
            
            "start" => { reset_start(&mut port) },
            
            "stop" | "exit" => { running = false },
            _ => ()
        } break; }
    }
}

fn ping(port: &mut Box<dyn SerialPort>) {
    let mut buf = [0u8];
    write_read(port, &[0x01], &mut buf);
    
    if buf[0] == 0xEE {
        println!("pong!")
    } else {
        println!("err: {:#04x}", buf[0]);
    }
}

fn program(port: &mut Box<dyn SerialPort>, movie: &mut Movie) {
    if movie.file_type == parsers::FileType::NONE {
        println!("You need to load a TAS first!");
        return;
    }
    
    let old_timeout = port.timeout();
    port.set_timeout(Duration::from_secs(60)).unwrap();
    
    let blocks = movie.to_blocks();
    
    port.write(&[0xAA]).unwrap(); // initiate programming sequence
    
    for (i, block) in blocks.iter().enumerate() {
        let mut sync_byte_buf = [0u8];
        port.read(&mut sync_byte_buf).unwrap(); // wait for block request
        if sync_byte_buf[0] != 0x01 {
            panic!("Programming sync byte mismatch, programmed data could be corrupt!");
        }
        port.write(&[0x01]).unwrap();
        
        let mut read_verify_buf = [0u8; 256];
        write_read(port, block, &mut read_verify_buf);  // write block and read block back from device
        if *block != read_verify_buf {                  // if block doesn't match original, USB corruption may have happened
            println!("Write/read mismatch! Block #{}", i);
        }
        
        if i > 0 && i % 16 == 0 {
            println!("blocks programmed/remaining: {}/{}", i, blocks.len());
        }
    }
    
    let mut sync_byte_buf = [0u8];
    port.read(&mut sync_byte_buf).unwrap(); // wait for block request
    port.write(&[0x00]).unwrap();
    
    port.clear(ClearBuffer::All).unwrap();
    port.set_timeout(old_timeout).unwrap();
    
    println!("Programming complete.")
}

fn everdrive_start(port: &mut Box<dyn SerialPort>) {
    let mut buf = [0u8];
    write_read(port, &[0x05], &mut buf);
    
    if buf[0] == 0xDD {
        println!("Acknowledged, press START button and immedately disconnect controller, to start TAS playback.")
    } else {
        println!("err: {:#04x}", buf[0]);
    }
}

fn reset_start(port: &mut Box<dyn SerialPort>) {
    let mut buf = [0u8];
    write_read(port, &[0x06], &mut buf);
    
    if buf[0] == 0xDD {
        println!("Acknowledged, playback started!")
    } else {
        println!("err: {:#04x}", buf[0]);
    }
}

fn write_read(port: &mut Box<dyn SerialPort>, write_buf: &[u8], read_buf: &mut [u8]) {
    port.write_all(write_buf).unwrap();
    port.read_exact(read_buf).unwrap();
}

fn load_port(use_first: bool) -> Option<Box<dyn SerialPort>> {
    let ports = serialport::available_ports().unwrap();
    
    if ports.is_empty() {
        println!("No ports detected!");
        return None;
    }
    
    if use_first {
        println!("Loading default port {}", ports.first().unwrap().port_name);
        return Some(serialport::new(ports.first().unwrap().port_name.clone(), 500000).timeout(Duration::from_secs(10)).open().expect("Failed to open port!"));
    }
    
    ports.iter().enumerate().for_each(|(i, info)| {
        println!("{}: {:?}", i, info);
    });
    print!("Choose port[0-{}]: ", (ports.len() - 1));
    std::io::stdout().flush().unwrap();
    let index: i8 = read_cli().parse().unwrap_or(-1);
    
    if index < 0 || index > (ports.len() - 1) as i8 {
        println!("Invalid option!");
        return None;
    }
    
    println!("Loading port #{} [{}]", index, ports[index as usize].port_name);
    
    Some(serialport::new(ports[index as usize].port_name.clone(), 500000).timeout(Duration::from_secs(10)).open().expect("Failed to open port!"))
}

fn read_cli() -> String {
    let mut cli_input = String::new();
    std::io::stdin().read_line(&mut cli_input).unwrap();
    
    cli_input.trim().to_string()
}
