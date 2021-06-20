use FileType::*;

#[allow(dead_code)]
#[derive(PartialEq, Debug, Clone)]
pub enum FileType {
    M64,
    R08, R16,
    NONE
}

#[derive(Debug, Clone)]
pub struct Movie {
    pub inputs: Vec<u8>,
    pub file_type: FileType,
}
impl Movie {
    pub fn new(file_type: FileType) -> Self {
        Movie {
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

pub fn parse(path: &str) -> Movie {
    let suffix = path.split(".").last().unwrap().to_ascii_lowercase();
    println!("SUFFIX: {}", suffix);
    let f = match suffix.as_str() {
        "m64" => M64,
        "r08" => R08,
        "r16" => R16,
        _ => { panic!("Unsupported file type! Ensure file name ends with proper file extension."); }
    };
    let mut movie = Movie::new(f);
    
    let bytes = std::fs::read(path).unwrap_or(vec![]);
    
    match movie.file_type {
        M64 | R16 => {
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
        NONE => ()
    }
    
    
    movie
}

fn div_up(a: usize, b: usize) -> usize {
    a / b + (a % b != 0) as usize
}