use std::fmt::{Display, Write};

//use log::{trace, warn};
use proc_bitfield::bitfield;

#[derive(Debug, Default, Copy, Clone)]
pub struct PI {
    pub PI_DRAM_ADDR: u32,
    pub PI_CART_ADDR: u32,
    pub PI_RD_LEN: u32,
    pub PI_WR_LEN: u32,
    pub PI_STATUS: u32,
    pub PI_BSD_DOM1_LAT: u32,
    pub PI_BSD_DOM2_LAT: u32,
    pub PI_BSD_DOM1_PWD: u32,
    pub PI_BSD_DOM2_PWD: u32,
    pub PI_BSD_DOM1_PGS: u32,
    pub PI_BSD_DOM2_PGS: u32,
    pub PI_BSD_DOM1_RLS: u32,
    pub PI_BSD_DOM2_RLS: u32,
}

#[derive(Debug)]
pub struct DMA_transfer_command {
    pub from: u32,
    pub to: u32,
    pub len: usize,
}
impl Display for DMA_transfer_command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = format!(
            "from: {:#x}, to: {:#x}, len: {:#x}",
            self.from, self.to, self.len
        );

        f.write_str(&msg)
    }
}

#[derive(Debug)]
pub struct PI_STATUS_READ {
    pub complete: bool,
    pub DMA_error: bool,
    pub IO_error: bool,
    pub DMA_busy: bool,
}

impl From<u32> for PI_STATUS_READ {
    fn from(value: u32) -> Self {
        Self {
            complete: ((value & 0b1000) >> 3) != 0,
            DMA_error: ((value & 0b100) >> 2) != 0,
            IO_error: ((value & 0b10) >> 1) != 0,
            DMA_busy: (value & 0b1) != 0,
        }
    }
}

impl PI {
    pub fn read(&self, addr: u32, len: usize) -> Result<Vec<u8>, String> {
        //trace!("in PI read: addr: {addr:#x}, len: {:?}", len);

        if !(0x0460_0000..=0x0460_0030).contains(&addr) {
            panic!("attempting to read to a non mmio-address in PI {addr:#x}")
        }
        if len != 4 {
            panic!("attempting to read more than 32 bits in PI. is this behavior you really really actually want?")
        }

        match addr {
            //PI_STATUS
            0x0460_0010 => {
                let val: u32 = self.PI_STATUS.into();
                return Ok(val.to_le_bytes().to_vec());
            }
            _ => {
                panic!("panicking in PI read on a bad address within PI MMIO range. probably have not implemented reading this reg yet")
            }
        }
    }

    pub fn write(&mut self, addr: u32, val: Vec<u8>) -> Option<DMA_transfer_command> {
        //trace!("in PI write: addr: {addr:#x}, val: {:?}", val);
        if !(0x0460_0000..=0x0460_0030).contains(&addr) {
            panic!("attempting to write to a non mmio-address in PI {addr:#x}")
        }
        if val.len() != 4 {
            panic!("attempting to write more than 32 bits in PI. is this behavior you really really actually want? {:#x} bytes",val.len())
        }
        match addr {
            0x0460_0000 => {
                self.PI_DRAM_ADDR =
                    val[0] as u32 | (val[1] as u32) << 8 | (val[2] as u32) << 16 | (0 as u32) << 24
            }
            0x0460_0004 => {
                self.PI_CART_ADDR = val[0] as u32
                    | (val[1] as u32) << 8
                    | (val[2] as u32) << 16
                    | (val[3] as u32) << 24
            }
            0x0460_0008 => {
                self.PI_RD_LEN =
                    val[0] as u32 | (val[1] as u32) << 8 | (val[2] as u32) << 16 | (0 as u32) << 24;

                return Some(DMA_transfer_command {
                    from: self.PI_DRAM_ADDR,
                    to: self.PI_CART_ADDR,
                    len: (self.PI_RD_LEN + 1) as usize,
                });
            }
            0x0460_000C => {
                self.PI_WR_LEN =
                    val[0] as u32 | (val[1] as u32) << 8 | (val[2] as u32) << 16 | (0 as u32) << 24;

                return Some(DMA_transfer_command {
                    from: self.PI_CART_ADDR,
                    to: self.PI_DRAM_ADDR,
                    len: (self.PI_WR_LEN + 1) as usize,
                });
            }
            _ => {
                unreachable!("YOU SHOULD NEVER GET HERE. HOW DID U GET HERE.")
            }
        }
        None
    }
}

/*
   2.1 0x0460 0000 - PI_DRAM_ADDR
   2.2 0x0460 0004 - PI_CART_ADDR
   2.3 0x0460 0008 - PI_RD_LEN
   2.4 0x0460 000C - PI_WR_LEN
   2.5 0x0460 0010 - PI_STATUS
   2.6 0x0460 00n4 - PI_BSD_DOMn_LAT
   2.7 0x0460 00n8 - PI_BSD_DOMn_PWD
   2.8 0x0460 00nC - PI_BSD_DOMn_PGS
   2.9 0x0460 00n0 - PI_BSD_DOMn_RLS
*/

/*bitfield! {
    pub struct PI_DRAM_ADDR(pub u32): Debug,FromRaw,IntoRaw,DerefRaw{
        pub zeros: u8 @ 24..=31,
        pub addr: u32 @ 0..=23
    }
}

bitfield! {
    pub struct PI_CART_ADDR(pub u32):Debug,FromRaw,IntoRaw,DerefRaw{
        pub zeros: u8 @ 24..=31,
        pub addr: u32 @ 0..=23
    }
}*/
