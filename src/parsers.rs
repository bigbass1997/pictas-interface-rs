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
    
    pub fn to_blocks(&self) -> Vec<[u8; 256]> {
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
        M64 => {
            unimplemented!();
        },
        R08 | R16 => {
            movie.inputs = bytes.clone();
            movie.inputs.iter_mut().for_each(|b| { *b ^= 0xFF });
        },
        NONE => ()
    }
    
    
    movie
}

fn div_up(a: usize, b: usize) -> usize {
    a / b + (a % b != 0) as usize
}