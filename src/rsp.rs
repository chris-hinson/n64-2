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
            //IMEM: [0; 4096],
            IMEM: Vec::with_capacity(4096),
            DMEM: Vec::with_capacity(4096),
        }
    }
}
