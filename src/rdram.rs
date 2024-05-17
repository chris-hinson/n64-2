use std::fmt::format;

//use log::debug;

pub struct Rdram {
    pub mem: Vec<u8>,
    RI_MODE: u32,
    RI_CONFIG: u32,
    RI_CURRENT_LOAD: u32,
    RI_SELECT: u32,
    RI_REFRESH: u32,
    RI_LATENCY: u32,
}

/*impl Rdram {
    pub fn new() -> Self {
        Rdram {
            mem: [0; 4608],
            RI_SELECT: 0x14,
            ..Default::default()
        }
    }
}*/

impl Default for Rdram {
    fn default() -> Self {
        return Rdram {
            //mem: Vec::with_capacity(4194304),
            mem: vec![0;4194304],
            RI_SELECT: 0x14,
            RI_MODE: 0,
            RI_CONFIG: 0,
            RI_CURRENT_LOAD: 0,
            RI_LATENCY: 0,
            RI_REFRESH: 0,
        };
    }
}

//read and writes come in from the cpu's addressing space
impl Rdram {
    pub fn read(&self, addr: u32, len: usize) -> Result<Vec<u8>, String> {
        if (0x0470_0000..=0x470_0014).contains(&addr) {
            //this is reading a RDRAM INTERFACE config register
            match addr {
                0x0470_0000 => Ok(self.RI_MODE.to_be_bytes().to_vec()),
                0x0470_00004 => Ok(self.RI_CONFIG.to_be_bytes().to_vec()),
                0x0470_00008 => Ok(self.RI_CURRENT_LOAD.to_be_bytes().to_vec()),
                0x0470_0000C => Ok(self.RI_SELECT.to_be_bytes().to_vec()),
                0x0470_00010 => Ok(self.RI_REFRESH.to_be_bytes().to_vec()),
                0x0470_00014 => Ok(self.RI_LATENCY.to_be_bytes().to_vec()),
                _ => unreachable!("how the fuck did u get here"),
            }
        } else if (0x0000_0000..=0x03FF_FFFF).contains(&addr) {
            //this is reading somewhere actually within rdram

            if (0x03F0_0000..=0x03F7_FFFF).contains(&addr)
                || (0x03F8_0000..=0x03FF_FFFF).contains(&addr)
            {
                unimplemented!(
                    "physical rdram register read access not implemented: {:#x}",
                    addr
                )
            }

            let final_addr = 0b0000000 << 29
                | ((addr >> 20) & 0x3F) << 20
                | ((addr >> 11) & 0x1FF) << 11
                | (addr & 0x7FF);

            return Ok(self.mem[(final_addr as usize)..(final_addr as usize) + len].to_vec());
        } else {
            Err(format!("trying to read a region not within RI {:#x}", addr).to_string())
        }
    }

    pub fn write(&mut self, addr: u32, data: Vec<u8>) -> Result<usize, String> {
        if (0x0470_0000..=0x470_0014).contains(&addr) {
            //this is writing a RDRAM INTERFACE config register
            match addr {
                0x0470_0000 => {
                    self.RI_MODE = (data[0] as u32
                        | (data[1] as u32) << 8
                        | (data[2] as u32) << 16
                        | (data[3] as u32) << 24) as u32;
                    Ok(32)
                }
                0x0470_00004 => {
                    self.RI_CONFIG = (data[0] as u32
                        | (data[1] as u32) << 8
                        | (data[2] as u32) << 16
                        | (data[3] as u32) << 24) as u32;
                    Ok(32)
                }
                0x0470_00008 => {
                    self.RI_CURRENT_LOAD = (data[0] as u32
                        | (data[1] as u32) << 8
                        | (data[2] as u32) << 16
                        | (data[3] as u32) << 24) as u32;
                    Ok(32)
                }
                0x0470_0000C => {
                    self.RI_SELECT = (data[0] as u32
                        | (data[1] as u32) << 8
                        | (data[2] as u32) << 16
                        | (data[3] as u32) << 24) as u32;
                    Ok(32)
                }
                0x0470_00010 => {
                    self.RI_REFRESH = (data[0] as u32
                        | (data[1] as u32) << 8
                        | (data[2] as u32) << 16
                        | (data[3] as u32) << 24) as u32;
                    Ok(32)
                }
                0x0470_00014 => {
                    self.RI_LATENCY = (data[0] as u32
                        | (data[1] as u32) << 8
                        | (data[2] as u32) << 16
                        | (data[3] as u32) << 24) as u32;
                    Ok(32)
                }
                _ => unreachable!("how the fuck did u get here"),
            }
        } else if (0x0000_0000..=0x03FF_FFFF).contains(&addr) {
            //this is writing somewhere actually within rdram
            if (0x03F0_0000..=0x03F7_FFFF).contains(&addr)
                || (0x03F8_0000..=0x03FF_FFFF).contains(&addr)
            {
                unimplemented!(
                    "physical rdram register write access not implemented: {:#x}",
                    addr
                )
            }

            let final_addr = 0b0000000 << 29
                | ((addr >> 20) & 0x3F) << 20
                | ((addr >> 11) & 0x1FF) << 11
                | (addr & 0x7FF);

            //debug!("in rdram::write, final_addr is: {:#x}", final_addr);

            //TODO: check your math here i have no idea if youre adding the proper offset lol
            unsafe {
                let src_ptr = data.as_ptr();
                let mut dst_ptr = self.mem.as_mut_ptr();
                dst_ptr = dst_ptr.add(final_addr as usize);
                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, data.len());
            }

            return Ok(data.len());
        } else {
            Err(format!("trying to write a region not within RI {:#x}", addr).to_string())
        }
    }
}
