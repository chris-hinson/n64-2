use std::{cell::RefCell, rc::Rc};

use disas::Disasembler;

use crate::cart::Cart;
use crate::cpu::{self, ICPU};
use crate::pi::PI;
use crate::rdram::Rdram;
use crate::rsp::Rsp;

pub struct System<'a> {
    pub disas: Disasembler<'a>,
    pub cpu: ICPU<'a>,
    pub pi: PI,
    pub rdram: Rdram,
    pub cart: Cart,
    pub rsp: Rsp,
}
//read and write
impl<'a> System<'a> {
    pub fn write(&mut self, addr: usize, bytes: &[u8]) -> bool {
        let pa = self.virt_to_phys(addr);
        match pa {
            //RCP PI, not PI external bus
            0x04600000..=0x046FFFFF => {
                let possible_dma = self.pi.write(pa as u32, bytes.to_vec());
                if possible_dma.is_some() {
                    //debug!("begin PI DMA");
                    let dma_packet = possible_dma.unwrap();
                    //debug!("PI DMA packet: {}", dma_packet);

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
        let pa = self.virt_to_phys(addr);
        //RCP PI, not PI external bus
        match pa {
            0x04600000..=0x046FFFFF => self.pi.read(pa as u32, len).unwrap(),
            _ => unimplemented!("dont have reads for addr:{:x} yet", addr),
        }
    }
    pub fn virt_to_phys(&self, virt: usize) -> usize {
        let modified_virt = virt as i32 as u32;
        match modified_virt {
            0x0000_0000..=0x7FFF_FFFF => {
                panic!("tried to convert a virtual address in KUSEG {virt:#x}")
            } //KUSEG
            0x8000_0000..=0x9FFF_FFFF => {
                panic!("tried to convert a virtual address in KSEG0 {virt:#x}")
            } //KSEG0
            0xA000_0000..=0xBFFF_FFFF => {
                //trace!("in system::virt_to_phys, virt is {:#x}", virt);

                let conversion = modified_virt.checked_sub(0xA000_0000);

                //trace!("after sub: {:?}", conversion);
                match conversion {
                    Some(v) => {
                        //trace!("value is {:#x}", v);
                        println!("VA: {:#016x}, PA: {:#016x}", virt, v);
                        return v as usize;
                    }
                    None => panic!("error converting address in KSEG1 {virt:#x}"),
                }
            } //KSEG1
            0xC000_0000..=0xDFFF_FFFF => {
                panic!("tried to convert a virtual address in KSEG2 {virt:#x}")
            } //KSEG2
            0xE000_0000..=0xFFFF_FFFF => {
                panic!("tried to convert a virtual address in KSEG3 {virt:#x}")
            } //KSEG3
            _ => {
                panic!("tried to convert a VA outside the range of a 32 bit unsigned integer")
            }
        }
    }
}

impl<'a> System<'a> {
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

        //copies the first 0x1000 bytes from the cartridge (located at 0xb000 0000)to memory address 0xA400 0000
        unsafe {
            let dst = self.rsp.DMEM.as_mut_ptr();
            let src = self.cart.rom.as_ptr();
            std::ptr::copy_nonoverlapping(src, dst, 0x1000);
        }
        self.cpu.rf.PC = 0xA4000040;
        self.disas.data = &mut self.rsp.DMEM as *mut Vec<u8>;
        self.disas.data_base_addr = 0x04000000;
    }

    pub fn run(&mut self) {
        loop {
            /*let fuckingchrist =
                self.virt_to_phys(self.cpu.rf.PC as usize) ;
            println!("fucking christ: {:x}", fuckingchrist);
            self.disas.find_basic_block(fuckingchrist);
            let the_block = self.disas.get_basic_block_at_addr*/



            /*for (k, block) in self.disas.Blocks.iter() {
                println!("block contains {} instructions", block.instrs.len());
                for instr in block.instrs.iter() {
                    println!("{}", instr);
                    (instr.operation)(&mut self.cpu, *instr.as_ref());

                    println!("{}", self.cpu.rf);
                }
                }*/

            //self.cpu.rf.PC = self.cpu.rf.PC +=
            //let the_block = self.disas.Blocks.get()
        }
    }

    pub fn find_basic_block(&mut self, addr:usize){
        self.disas.find_basic_block(self.cpu.rf.PC);
        while self.
    }
}

impl<'a> System<'a> {
    pub fn new(d: Disasembler<'a>, cart: Cart) -> Self {
        System {
            disas: d,
            pi: PI::default(),
            rdram: Rdram::default(),
            cpu: crate::cpu::ICPU::new(),
            rsp: Rsp::default(),
            cart,
        }
    }
}
