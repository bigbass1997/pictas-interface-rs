use FileType::*;
use tasd_edit::movie::TasdMovie;
use std::path::PathBuf;
use tasd_edit::definitions::{INPUT_CHUNKS, InputChunks};

#[allow(dead_code)]
#[derive(PartialEq, Debug, Clone)]
pub enum FileType {
    TASD,
    M64,
    R08,
    R16M,
    A2600Bin,
    NONE
}

#[derive(Debug, Clone)]
pub struct InputsMovie {
    pub inputs: Vec<u8>,
    pub file_type: FileType,
}
impl InputsMovie {
    pub fn new(file_type: FileType) -> Self {
        InputsMovie {
            inputs: Vec::new(),
            file_type: file_type,
        }
    }
    
    pub fn none() -> Self {
        Self::new(NONE)
    }
    
    pub fn prepend(&mut self, bytes: &[u8], invert: bool){
        for b in bytes.iter().rev() {
            let mut bb = *b;
            if invert {
                bb ^= 0xFF;
            }
            
            self.inputs.insert(0, bb);
        }
    }
    
    pub fn input_blocks(&self) -> Vec<[u8; 256]> {
        let mut blocks: Vec<[u8; 256]> = Vec::new();
        
        let count = div_up(self.inputs.len(), 256);
        for i in 0..count {
            let mut block = [0u8; 256];
            for j in 0..256 {
                let index = (i * 256) + j;
                let mut byte = 0xFF;
                if index < self.inputs.len() {
                    byte = self.inputs[index];
                }
                
                block[j] = byte;
            }
            
            blocks.push(block);
        }
        
        blocks
    }
    
    pub fn config_blocks(&self) -> [[u8; 256]; 16] {
        let mut blocks = [[0xFF; 256]; 16];
        
        //todo!() config stuff
        for cfg_byte in blocks[0].iter_mut() {
            *cfg_byte = 0;
        }
        
        let mut last_block_index = 1;
        let mut last_byte_index = 0;
        for i in 0..(self.inputs.len() / 2) {
            let i2 = i * 2;
            let input = self.inputs.get(i2).unwrap();
            
            if *input == 0x01 {
                let f = i as u32;
                let high = ((f >> 16) & 0xFF) as u8;
                let mid  = ((f >>  8) & 0xFF) as u8;
                let low  = ((f >>  0) & 0xFF) as u8;
                blocks[last_block_index][last_byte_index + 0] = high;
                blocks[last_block_index][last_byte_index + 1] = mid;
                blocks[last_block_index][last_byte_index + 2] = low;
                blocks[last_block_index][last_byte_index + 3] = 0x01;
                
                println!("Reset at frame: {} ({:02X} {:02X} {:02X})", f, high, mid, low);
                println!("Block ID: [{}][{}]", last_block_index, last_byte_index);
                
                last_byte_index += 4;
                if last_byte_index >= 256 {
                    last_block_index += 1;
                    if last_block_index >= 16 {
                        panic!("Movie contains too many resets!");
                    }
                }
            }
        }
        
        blocks
    }
}

pub fn parse(path: &str) -> InputsMovie {
    let suffix = path.split(".").last().unwrap().to_ascii_lowercase();
    println!("SUFFIX: {}", suffix);
    let f = match suffix.as_str() {
        "tasd" => TASD,
        "m64" => M64,
        "r08" => R08,
        "r16m" => R16M,
        "bin" => A2600Bin,
        _ => { panic!("Unsupported file type! Ensure file name ends with proper file extension."); }
    };
    let mut movie = InputsMovie::new(f);
    
    let bytes = std::fs::read(path.clone()).unwrap_or(vec![]);
    
    match movie.file_type {
        TASD => {
            //TODO: convert entire project to only support TASD
            
            // Assumes data is NES, 2 controllers.
            let tasd = TasdMovie::new(&PathBuf::from(path)).unwrap();
            let search = tasd.search_by_key(vec![INPUT_CHUNKS]);
            
            let mut port1 = Vec::new();
            let mut port2 = Vec::new();
            for packet in search {
                let packet = packet.as_any().downcast_ref::<InputChunks>().unwrap();
                if packet.port == 1 { packet.payload.iter().for_each(|byte| port1.push(*byte)) }
                if packet.port == 2 { packet.payload.iter().for_each(|byte| port2.push(*byte)) }
            }
            
            for i in 0..port1.len() {
                movie.inputs.push(port1[i]);
                movie.inputs.push(port2[i]);
            }
            
            for _ in 0..48000 {
                movie.inputs.push(0xFF); // temporary until PICTAS properly stops at the end of a TAS.
            }
        },
        M64 | R16M => {
            unimplemented!();
        },
        R08 => {
            movie.inputs = bytes.clone();
            
            /*let mut indices = Vec::new();
            for (i, input) in movie.inputs.iter().enumerate() {
                if *input == 0xFE && *movie.inputs.get(i + 1).unwrap_or(&0xFF) == 0x00 {
                    indices.push(i + 1);
                }
            }
            indices.reverse();
            for i in indices {
                movie.inputs.remove(i);
            }*/
            
            movie.inputs.iter_mut().for_each(|b| { *b ^= 0xFF });
        },
        A2600Bin => {
            movie.inputs = bytes.clone();
        },
        NONE => ()
    }
    
    
    movie
}

fn div_up(a: usize, b: usize) -> usize {
    a / b + (a % b != 0) as usize
}
