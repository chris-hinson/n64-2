#[derive(Debug,Default,Clone, Copy)]
pub struct SI{
    SI_DRAM_ADDR: u32,
    SI_PIF_AD_RD64B : u32,
    SI_PIF_AD_WR4B : u32,
    SI_PIF_AD_WR64B : u32,
    SI_PIF_AD_RD4B : u32,
    SI_STATUS : u32
}

/*    
SI_DRAM_ADDR 0x0480 0000 
SI_PIF_AD_RD64B 0x0480 0004 
SI_PIF_AD_WR4B 0x0480 0008 
SI_PIF_AD_WR64B 0x0480 0010 
SI_PIF_AD_RD4B 0x0480 0014 
SI_STATUS 0x0480 0018 
*/

impl SI {
    pub fn read(&mut self, addr: usize, len: usize) -> Vec<u8> {
        match addr {
            _=>panic!("bad paddr in SI read {:x}",addr)
        }
    }
    pub fn write(&mut self, addr: usize, bytes: &[u8]) -> bool {

        //if we end up writing a byte into the command space
        if addr + bytes.len() >= 0x1FC007FF{
            let addresses_iter = (addr..=addr+bytes.len()).into_iter();
            let bytes_iter = bytes.into_iter().rev();
            let mut addr_bytes_pairs = addresses_iter.zip(bytes_iter);

            let command_byte = addr_bytes_pairs.find(|x| x.0 == 0x1FC007FF).unwrap().1;

            match command_byte {
                0x08 =>{} //Terminate boot process. just nop this for now
                _=> panic!("pif RAM received command byte we havent implemented yet")
            }

            return true;
        }

        match addr {
            _=>panic!("bad paddr in SI write {:x}",addr)
        }
    }
}