mod cpu;
mod system;
use asm::Disasembler;
use cart::Cart;
use system::System;
mod asm;
mod cart;
mod pi;
mod rdram;
mod rsp;
mod si;
fn main() {
    let cart = Cart::new("./addiu_simpleboot.z64");
    //let d = disas::Disasembler::new(disas_requester,disas_reader);
    //let d = disas::Disasembler::new(std::ptr::null_mut::<Vec<u8>>(), 0);
    let d = Disasembler::new();

    let mut sys = system::System::new(d, cart);
    let raw_sys_ptr = &mut sys as *mut System;
    sys.cpu.parent = raw_sys_ptr;
    sys.disas.system = raw_sys_ptr;

    //run bootrom
    sys.boot();

    //assert_eq!(sys.disas.data, &mut sys.rsp.DMEM as *mut Vec<u8>);

    sys.run();
}
