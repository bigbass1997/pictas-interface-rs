
use serialport::{SerialPort, ClearBuffer};
use std::time::Duration;
use std::io::{Write, Read};
use crate::parsers::InputsMovie;
use crate::parsers::FileType::NONE;

macro_rules! buffer {
    ($name:ident, $val:expr) => {
        pub const $name: &[u8] = &[$val];
    };
}

macro_rules! byte {
    ($name:ident, $val:expr) => {
        pub const $name: u8 = $val;
    };
}

buffer!(BLANK,              0x00);
buffer!(CMD_PING,           0x01);
buffer!(CMD_RESET_START,    0x06);
buffer!(CMD_PROGRAM_INPUTS, 0xAA);
buffer!(CMD_PROGRAM_CONFIG, 0xAB);
buffer!(NO_INPUTS_REMAIN,   0x00);

byte!(RES_OK,               0xEE);
byte!(PROGRAM_SYNC,         0x01);
byte!(PROGRAMMING_DONE,     0xDD);
byte!(ACKNOWLEDGED,         0xDD);





pub struct Device {
    pub port: Option<Box<dyn SerialPort>>,
}
impl Device {
    pub fn new() -> Self {
        Self {
            port: None,
        }
    }
    
    pub fn ping(&mut self) -> Result<String, u8> {
        self.port.as_mut().unwrap().clear(ClearBuffer::All).unwrap();
        let mut buf = [0u8];
        Self::write_read(self.port.as_mut().unwrap(), CMD_PING, &mut buf);
        
        if buf[0] == RES_OK {
            Result::Ok(String::from("pong!"))
        } else {
            Result::Err(buf[0])
        }
    }
    
    pub fn program(&mut self, movie: &InputsMovie) -> Result<(), String> {
        if self.port.is_none() {
            return Err(String::from("Not connected to device."));
        }
        
        let inputs_result = self.program_inputs(movie);
        match inputs_result {
            Ok(_) => (),
            Err(_) => return inputs_result.clone(),
        }
        
        self.program_config(movie)
    }
    
    pub fn program_inputs(&mut self, movie: &InputsMovie) -> Result<(), String> {
        if movie.file_type == NONE {
            return Err(String::from("No TAS loaded!"));
        }
        
        let port = self.port.as_mut().unwrap();
        port.clear(ClearBuffer::All).unwrap();
        
        let old_timeout = port.timeout();
        port.set_timeout(Duration::from_secs(60)).unwrap();
        
        port.write_all(CMD_PROGRAM_INPUTS).unwrap();
        
        let blocks = movie.input_blocks();
        for (i, block) in blocks.iter().enumerate() {
            let mut sync_byte_buf = [0u8];
            port.read_exact(&mut sync_byte_buf).unwrap(); // wait for block request
            if sync_byte_buf[0] != PROGRAM_SYNC {
                return Err(String::from("Programming sync byte mismatch, programmed data could be corrupt!"));
            }
            port.write_all(&[PROGRAM_SYNC]).unwrap(); // write sync byte back
            
            let mut read_verify_buf = [0u8; 256];
            Self::write_read(port, block, &mut read_verify_buf); // write data block
            if *block != read_verify_buf {
                println!("Write/read mismatch! Block #{}", i);
            }
            
            let j = i + 1;
            if j > 0 && j % 16 == 0 {
                println!("blocks programmed/total: {}/{}", j, blocks.len());
            }
        }
        
        port.read_exact(&mut [0u8]).unwrap();
        port.write_all(NO_INPUTS_REMAIN).unwrap();
        
        port.clear(ClearBuffer::All).unwrap();
        port.set_timeout(old_timeout).unwrap();
        
        println!("TAS programming complete.");
        
        Ok(())
    }
    
    pub fn program_config(&mut self, movie: &InputsMovie) -> Result<(), String> {
        if movie.file_type == NONE {
            return Err(String::from("No TAS loaded!"));
        }
        
        let port = self.port.as_mut().unwrap();
        port.clear(ClearBuffer::All).unwrap();
        
        let old_timeout = port.timeout();
        port.set_timeout(Duration::from_secs(60)).unwrap();
        
        port.write_all(CMD_PROGRAM_CONFIG).unwrap();
        
        let blocks = movie.config_blocks();
        for (i, block) in blocks.iter().enumerate() {
            let mut sync_byte_buf = [0u8];
            port.read_exact(&mut sync_byte_buf).unwrap(); // wait for block request
            if sync_byte_buf[0] != PROGRAM_SYNC {
                return Err(String::from("Programming sync byte mismatch, programmed data could be corrupt!"));
            }
            port.write_all(&[PROGRAM_SYNC]).unwrap(); // write sync byte back
            
            let mut read_verify_buf = [0u8; 256];
            Self::write_read(port, block, &mut read_verify_buf); // write data block
            if *block != read_verify_buf {
                println!("Write/read mismatch! Block #{}", i);
            }
            
            let j = i + 1;
            if j > 0 && j % 4 == 0 {
                println!("blocks programmed/total: {}/{}", j, blocks.len());
            }
        }
        
        let mut sync_byte_buf = [0u8];
        port.read_exact(&mut sync_byte_buf).unwrap();
        
        port.clear(ClearBuffer::All).unwrap();
        port.set_timeout(old_timeout).unwrap();
        
        if sync_byte_buf[0] != PROGRAMMING_DONE {
            return Err(String::from(format!("Config programming failed: {:#04X}", sync_byte_buf[0])));
        }
        
        println!("Config programming complete.");
        
        return Ok(())
    }
    
    pub fn reset_start(&mut self) -> Result<String, u8> {
        let mut buf = [0u8];
        Self::write_read(self.port.as_mut().unwrap(), CMD_RESET_START, &mut buf);
        
        if buf[0] == ACKNOWLEDGED {
            Result::Ok(String::from("Acknowledged, playback started!"))
        } else {
            Result::Err(buf[0])
        }
    }
    
    pub fn atari_start(&mut self) -> Result<String, u8> {
        let mut buf = [0u8];
        Self::write_read(self.port.as_mut().unwrap(), &[0x08], &mut buf);
        
        if buf[0] == ACKNOWLEDGED {
            Result::Ok(String::from("Acknowledged, playback started!"))
        } else {
            Result::Err(buf[0])
        }
    }
    
    
    pub fn cli_select_port(&mut self, use_first: bool) -> bool {
        let ports_result = serialport::available_ports();
        
        if ports_result.is_ok() {
            let ports = ports_result.unwrap();
            
            if ports.is_empty() {
                return false;
            }
            
            if use_first {
                println!("Loading default port {}", ports.first().unwrap().port_name);
                self.port = Some(serialport::new(ports.first().unwrap().port_name.clone(), 500000).timeout(Duration::from_secs(30)).open().expect("Failed to open port!"));
                
                return true;
            }
            
            ports.iter().enumerate().for_each(|(i, info)| {
                println!("{}: {:?}", i, info);
            });
            print!("Choose port[0-{}]: ", (ports.len() - 1));
            std::io::stdout().flush().unwrap();
            let index: isize = Self::read_cli().parse().unwrap_or(-1);
            
            if index < 0 || index > (ports.len() - 1) as isize {
                println!("Invalid option!");
                return false;
            }
            
            println!("Loading port #{} [{}]", index, ports[index as usize].port_name);
            self.port = Some(serialport::new(ports[index as usize].port_name.clone(), 500000).timeout(Duration::from_secs(30)).open().expect("Failed to open port!"));
            
            return true;
        }
        
        false
    }
    
    fn read_cli() -> String {
        let mut cli_input = String::new();
        std::io::stdin().read_line(&mut cli_input).unwrap();
        
        cli_input.trim().to_string()
    }
    
    pub fn write_read(port: &mut Box<dyn SerialPort>, write_buf: &[u8], read_buf: &mut [u8]) {
        port.write_all(write_buf).unwrap();
        port.read_exact(read_buf).unwrap();
    }
}
