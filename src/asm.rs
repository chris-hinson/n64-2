use disas::instr::Instruction;
use disas::instr::INSTR_WHICH_END_BASIC_BLOCK;
use std::{collections::HashMap, rc::Rc};

#[allow(non_snake_case)]
#[derive(Clone)]
pub struct Disasembler {
    pub system: *mut crate::system::System,

    //hashset of thus far decoded instructions
    //this is the "global" instruction translation cache
    //we hash the raw bits of an instruction and equate that with a fully formed Instruction struct
    //TODO: hashing is overkill here. use a 2 or 3 tier linear cache that populates pages as we hit them (sparse array)
    pub ITC: HashMap<u32, Rc<Instruction>>,

    //basic blocks are indexed by their base address only.
    //this DOES mean that we may at some point try and jump into the middle of an existing basic block and not realize
    //but we really dont give a shit because we can just make a new basic block, and since all instructions are RCed, we can include instructions in multiple blocks
    pub Blocks: HashMap<usize, BasicBlock>,
}

impl Disasembler {
    pub fn new() -> Self {
        Self {
            ITC: HashMap::new(),
            Blocks: HashMap::new(),
            system: std::ptr::null_mut(),
        }
    }

    //finding a basic block differs in interpreter and JIT mode
    //in interpreter mode, a basic block is defined by as many instructions as you can go without hitting a control flow op
    //in a jit block, the same rule holds BUT we may need to stop the block early if we run out of host registers (especially prevalent on x86)
    //NOTE: maybe we just always find it in the "interpreter" way since thats the more textbook definition of a basic block
    //and then in jit mode, if we run out of registers WHEN EMITTING we can split the basic block and error handle in the emitter
    pub fn find_basic_block(&mut self, mut addr: usize) {
        //HACK
        //turn the PA into a VA in the uncached non-tlb seg so we can go through the system read function

        let base_addr = addr;
        let mut cur_block = BasicBlock {
            _valid: true,
            base: addr,
            instrs: Vec::new(),
        };

        println!("[disas] starting to find basic block at addr {:#04x}", addr);

        //grab some bytes and turn them into an instruction
        let mut cur_bytes = unsafe { (*self.system).read(base_addr + 0xA0000000, 4) };
        let mut cur_instr = disas::decode(
            u32::from_be_bytes(cur_bytes[0..4].try_into().unwrap()),
            false,
        );
        addr += 4;

        //did we just decode a block ending instruction?
        while !INSTR_WHICH_END_BASIC_BLOCK.contains(&cur_instr.opcode) {
            //println!("opcode: {:?}", cur_instr.opcode);
            cur_block.instrs.push(Rc::new(cur_instr));

            cur_bytes = unsafe { (*self.system).read(addr + 0xA0000000, 4) };
            cur_instr = disas::decode(
                u32::from_be_bytes(cur_bytes[0..4].try_into().unwrap()),
                false,
            );
            addr += 4;
        }

        //ADD THE DELAY SLOT INSTRUCTION
        /*if cur_instr.opcode != ERET {
            //push the last instr we decoded before we figure out our block was over
            cur_block.instrs.push(cur_instr);
            cur_bytes = (self.Reader)(addr);
            cur_instr = self.decode(cur_bytes, true);
            cur_block.instrs.push(cur_instr);

            //TODO:HANDLE DELAY SLOT FUCKERY(nested delay slots)
        } else {*/
        //push the last instr we decoded before we figure out our block was over
        cur_instr.delay_slot = true;
        cur_block.instrs.push(Rc::new(cur_instr));
        //}

        self.Blocks.insert(base_addr, cur_block);


    }
}

//a basic block is a set of instructions.
//these instructions may either be interpreted or emitted into a host buffer to be executed by the jit engine
//this means that we must represent two possible
#[derive(Clone)]
pub struct BasicBlock {
    //is this block currently valid?
    _valid: bool,

    //base address of the block: PHYSICAL ADDRESS
    base: usize,

    //each instruction in a basic block is actually a pointer to that opcode in the global instruction translation cache (ITC)
    pub instrs: Vec<Rc<Instruction>>,
}

impl std::fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f,"┌─────────────┐")?;
        for (offset,op) in self.instrs.iter().enumerate(){
            writeln!(f,"│{:#08x},{:?}│",self.base + offset,op.opcode)?;
        }
        writeln!(f,"└─────────────┘")
    }
}