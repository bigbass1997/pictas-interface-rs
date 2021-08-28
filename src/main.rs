

mod parsers;
mod device;

use parsers::InputsMovie;
use serialport::ClearBuffer;
use crate::device::Device;

fn main() {
    let mut device = Device::new();
    device.cli_select_port(true);
    
    let mut loaded_movie = InputsMovie::none();
    
    let mut running = true;
    while running {
        let cli_input = read_cli();
        let cli_parts: Vec<&str> = cli_input.split(" ").collect();
        
        loop { match cli_parts[0] {
            "clear" => {
                if device.port.is_some() {
                    device.port.as_mut().unwrap().clear(ClearBuffer::All).unwrap();
                }
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
            
            "port" => { device.cli_select_port(false); },
            
            "ping" => { match device.ping() {
                Ok(val) => println!("{}", val),
                Err(val) => println!("err: {:#04X}", val),
            };},
            
            "program" => { match device.program(&mut loaded_movie) {
                Ok(_) => (),
                Err(val) => println!("{}", val),
            };},
            "config" => { match device.program_config(&mut loaded_movie) {
                Ok(_) => (),
                Err(val) => println!("{}", val),
            };},
            
            //"everdrive" => { everdrive_start(port) },
            
            "start" => { match device.reset_start() {
                Ok(_) => (),
                Err(val) => println!("{}", val),
            };},
            
            //"manual" => { manual_start(port) },
            
            //"dump" => { dump(port) },
            
            "stop" | "exit" => { running = false },
            _ => ()
        } break; }
    }
}

/*
fn everdrive_start(port: UsbPort) {
    let mut buf = [0u8];
    write_read(port, &[0x05], &mut buf);
    
    if buf[0] == 0xDD {
        println!("Acknowledged, press START button and immedately disconnect controller, to start TAS playback.")
    } else {
        println!("err: {:#04x}", buf[0]);
    }
}


fn manual_start(port: UsbPort) {
    let mut buf = [0u8];
    write_read(port, &[0x07], &mut buf);
    
    if buf[0] == 0xDD {
        println!("Acknowledged, press console reset button to start TAS playback.")
    } else {
        println!("err: {:#04x}", buf[0]);
    }
}

fn write_read(port: UsbPort, write_buf: &[u8], read_buf: &mut [u8]) {
    port.write_all(write_buf).unwrap();
    port.read_exact(read_buf).unwrap();
}


fn dump(port: UsbPort) {
    //let vec: Vec<u8> = Vec::with_capacity(16*1024*1024);
    //let mut buf = vec.into_boxed_slice();
    let mut buf = [0u8; 4];
    write_read(port, &[0x02], &mut buf);
    
    std::fs::write(Path::new("dump.bin"), buf).unwrap();
    println!("Done");
    println!("{:02X} {:02X} {:02X} {:02X}", buf[0], buf[1], buf[2], buf[3]);
}*/

fn read_cli() -> String {
    let mut cli_input = String::new();
    std::io::stdin().read_line(&mut cli_input).unwrap();
    
    cli_input.trim().to_string()
}
