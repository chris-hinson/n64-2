use std::cell::RefCell;
use std::collections::HashMap;
mod cpu;
mod system;
use std::fs;
use std::fs::File;
use std::io::Read;

use cart::Cart;
use system::System;
mod cart;
mod pi;
mod rdram;
mod rsp;

fn main() {
    let mut cart = Cart::new("./addiu_simpleboot.z64");
    let d = disas::Disasembler::new(&mut cart.rom as *mut Vec<u8>, 0);
    //let d = disas::Disasembler::new(std::ptr::null_mut::<Vec<u8>>(), 0);

    let mut sys = system::System::new(d, cart);
    let raw_sys_ptr = &mut sys as *mut System;
    sys.cpu.parent = raw_sys_ptr;

    //run bootrom
    sys.boot();

    assert_eq!(sys.disas.data, &mut sys.rsp.DMEM as *mut Vec<u8>);

    sys.run();
}
