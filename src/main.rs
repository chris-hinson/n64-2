use std::cell::RefCell;
use std::collections::HashMap;
mod cpu;
mod system;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc;

use cart::Cart;
use system::System;
mod cart;
mod pi;
mod rdram;
mod rsp;

fn main() {
    let cart = Cart::new("./addiu_simpleboot.z64");
    let (disas_requester,emu_requester) = mpsc::channel::<u32>();
    let (emu_reader,disas_reader) = mpsc::channel::<u32>();
    let d = disas::Disasembler::new(disas_requester,disas_reader);
    //let d = disas::Disasembler::new(std::ptr::null_mut::<Vec<u8>>(), 0);

    let mut sys = system::System::new(d, emu_requester,emu_reader,cart);
    let raw_sys_ptr = &mut sys as *mut System;
    sys.cpu.parent = raw_sys_ptr;

    //run bootrom
    sys.boot();

    //assert_eq!(sys.disas.data, &mut sys.rsp.DMEM as *mut Vec<u8>);

    sys.run();
}
