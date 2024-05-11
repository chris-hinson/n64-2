#[derive(Debug)]
pub struct Rsp {
    //there should be a cpu here....
    //IMEM 4Kb
    //pub IMEM: [u8; 4096],
    pub IMEM: Vec<u8>,
    //DMEM 4Kb
    //pub DMEM: [u8; 4096],
    pub DMEM: Vec<u8>,
}
impl Default for Rsp {
    fn default() -> Self {
        Rsp {
            //DMEM: Vec::with_capacity(4096),
            DMEM: vec![0; 4096],
            IMEM: vec![0; 4096],
        }
    }
}
impl Rsp {
    pub fn read(&mut self, addr: usize, len: usize, IorD: bool) -> Vec<u8> {
        if !IorD {
            //DMEM
            return self.DMEM[(addr - 0x04000000)..(addr + len) - 0x04000000].to_vec();
        } else {
            //IMEM
            return self.IMEM[(addr - 0x04000000)..(addr + len) - 0x04000000].to_vec();
        }
    }
    pub fn write(&mut self, addr: usize, bytes: &[u8], IorD: bool) -> bool {
        todo!("can you even write to IMEM/DMEM from user code?")
    }
}
