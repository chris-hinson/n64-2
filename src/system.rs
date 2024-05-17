use std::sync::mpsc::{Receiver, Sender};
use std::{cell::RefCell, rc::Rc};

//use disas::{BasicBlock, Disasembler};

use std::thread::{self, JoinHandle};

use colored::Colorize;

use crate::asm::Disasembler;
use crate::cart::Cart;
use crate::cpu::{self, ICPU};
use crate::pi::PI;
use crate::rdram::Rdram;
use crate::rsp::Rsp;

pub struct System {
    pub disas: Disasembler,
    pub cpu: ICPU,
    pub pi: PI,
    pub rdram: Rdram,
    pub cart: Cart,
    pub rsp: Rsp,
}
//read and write
impl System {
    pub fn write(&mut self, addr: usize, bytes: &[u8]) -> bool {
        let pa = self.virt_to_phys(addr);
        match pa {
            //RDRAM
            0x0000_0000..=0x03FFFFFF =>{
                self.rdram.write(addr as u32, bytes.to_vec()).unwrap();
            }
            //RCP PI, not PI external bus
            0x04600000..=0x046FFFFF => {
                let possible_dma = self.pi.write(pa as u32, bytes.to_vec());
                if possible_dma.is_some() {
                    self.log("begin PI DMA".yellow());
                    let dma_packet = possible_dma.unwrap();
                    self.log(format!("PI DMA packet: {}", dma_packet).yellow());

                    //execute the dma
                    let len = dma_packet.len;
                    let from = dma_packet.from;
                    let to = dma_packet.to;

                    //coming FROM cart to rdram
                    if dma_packet.from >= 0x10000000 {
                        let data = self.cart.rom[(from - 0x10000000) as usize
                            ..((from - 0x10000000) as usize + len) as usize]
                            .to_vec();
                        self.rdram.write(to, data).unwrap();
                    }
                    //coming FROM rdram to cart
                    else {
                        //to_ptr = self.cart.borrow_mut().rom.as_mut_ptr();
                        let mut data = self.rdram.read(from, len).unwrap();
                        //self.
                        unsafe {
                            let mut to_ptr = self.cart.rom.as_mut_ptr();
                            to_ptr = to_ptr.add(to as usize);

                            let from_ptr = data.as_mut_ptr();

                            std::ptr::copy_nonoverlapping(from_ptr, to_ptr, len);
                        }
                    }
                }
            }
            
            
            _ => panic!("trying to write to a PA we have not mapped yet {:#08x}", pa),
        }
        true
    }
    pub fn read(&mut self, addr: usize, len: usize) -> Vec<u8> {
        //println!("system read at addr {:#x}, len {:#x}", addr, len);
        let pa = self.virt_to_phys(addr);
        match pa {
            0x0000_0000..=0x03FFFFFF => self.rdram.read(pa as u32, len).unwrap(),
            //RCP DMEM
            0x04000000..=0x04000FFF => self.rsp.read(pa, len, false),
            //RCP IMEM
            0x04001000..=0x04001FFF => self.rsp.read(pa, len, true),
            //RCP PI, not PI external bus
            0x04600000..=0x046FFFFF => self.pi.read(pa as u32, len).unwrap(),
            _ => unimplemented!("dont have reads for addr:{:x} yet", addr),
        }
    }
    pub fn virt_to_phys(&self, virt: usize) -> usize {
        let modified_virt = virt as i32 as u32;
        match modified_virt {
            0x0000_0000..=0x7FFF_FFFF => {
                //panic!("tried to convert a virtual address in KUSEG {modified_virt:#x}")
                return modified_virt as usize;
            } //KUSEG
            0x8000_0000..=0x9FFF_FFFF => {
                //panic!("tried to convert a virtual address in KSEG0 {modified_virt:#x}")
                //trace!("in system::virt_to_phys, virt is {:#x}", virt);

                let conversion = modified_virt.checked_sub(0x8000_0000);

                //trace!("after sub: {:?}", conversion);
                match conversion {
                    Some(v) => {
                        //trace!("value is {:#x}", v);
                        //println!("VA: {:#016x}, PA: {:#016x}", virt, v);
                        return v as usize;
                    }
                    None => panic!("error converting address in KSEG1 {modified_virt:#x}"),
                }
            } //KSEG0
            0xA000_0000..=0xBFFF_FFFF => {
                //trace!("in system::virt_to_phys, virt is {:#x}", virt);

                let conversion = modified_virt.checked_sub(0xA000_0000);

                //trace!("after sub: {:?}", conversion);
                match conversion {
                    Some(v) => {
                        //trace!("value is {:#x}", v);
                        //println!("VA: {:#016x}, PA: {:#016x}", virt, v);
                        return v as usize;
                    }
                    None => panic!("error converting address in KSEG1 {modified_virt:#x}"),
                }
            } //KSEG1
            0xC000_0000..=0xDFFF_FFFF => {
                panic!("tried to convert a virtual address in KSEG2 {modified_virt:#x}")
            } //KSEG2
            0xE000_0000..=0xFFFF_FFFF => {
                panic!("tried to convert a virtual address in KSEG3 {modified_virt:#x}")
            } //KSEG3
            _ => {
                panic!("tried to convert a VA outside the range of a 32 bit unsigned integer")
            }
        }
    }
}

impl System {
    pub fn boot(&mut self) {
        //who fucking knows if this is actually all the side effects. just boot me and go idgaf
        self.cpu.rf.gprs[20] = 0x1;
        self.cpu.rf.gprs[22] = 0x3f;
        self.cpu.rf.gprs[29] = 0xA4001FF0;

        self.cpu.cop0.Random = 0x0000001F;
        self.cpu.cop0.Status = 0x70400004.into();
        self.cpu.cop0.PRId = 0x00000B00.into();
        self.cpu.cop0.Config = 0x0006E463.into();
        //this goes to MI, which we havent bothered with yet, do it later
        //writes the value 0x01010101 to memory address 0x0430 0004.

        //essentially fake ipl2
        //copies the first 0x1000 bytes from the cartridge (located at 0xb000 0000)to memory address 0xA400 0000
        unsafe {
            let dst = self.rsp.DMEM.as_mut_ptr();
            let src = self.cart.rom.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 0x1000);
        }
        self.cpu.rf.PC = 0xA4000040;
        //self.disas.data = &mut self.rsp.DMEM as *mut Vec<u8>;
        //self.disas.data_base_addr = 0x04000000;
    }

    pub fn run(&mut self) {
        loop {
            let pc_virt_addr = self.virt_to_phys(self.cpu.rf.PC as usize);
            self.disas.find_basic_block(pc_virt_addr);
            let bb = self.disas.Blocks.get(&pc_virt_addr).unwrap();
            println!("found basic block \n{}",bb);
            for instr in bb.instrs.iter() {
                print!("{}",format!("PC: {:#x}, \n\t{}", self.cpu.rf.PC, *instr).green());
                (instr.operation)(&mut self.cpu, **instr);
                self.cpu.rf.PC += 4;
                //self.cpu.log(&format!("{}",self.cpu.rf));
            }
            println!("{}",self.cpu.rf);
        }
    }
}

impl System {
    pub fn new(d: Disasembler, cart: Cart) -> Self {
        System {
            disas: d,
            pi: PI::default(),
            rdram: Rdram::default(),
            cpu: crate::cpu::ICPU::new(),
            rsp: Rsp::default(),
            cart,
        }
    }

    pub fn log(&self, msg:colored::ColoredString){
        println!("{msg}");
    }
}
