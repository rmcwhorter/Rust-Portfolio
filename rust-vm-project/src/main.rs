extern crate num;
#[macro_use]
extern crate num_derive;
use anyhow::{anyhow, Result};

#[macro_use]
extern crate timeit;

#[derive(FromPrimitive, Debug, Copy, Clone)]
enum Instruction {
    Exit,              // ext - the computer does nothing, it just dies
    LoadFromMem,       // lod <mem_address> <register>
    WriteToMem,        // wrt <register> <mem_address>
    Add,               // add <reg1> <reg2> <reg3> - adds reg1 to reg2 and writes to reg3
    Sub,               // sub <reg1> <reg2> <reg3> - subs reg1 to reg2 and writes to reg3
    SetProgramCounter, // spc <u32_value> - sets the program counter to the u32 in the instruction
    ClearAllRegisters, // clra - sets every register to zero
    ClearRegister,     // clr <reg_addr> - sets this register to zero
    RegisterWrite,     // rw <reg1> <reg2> - writes the value of register 1 to register 2
    IfEqSPCElsePass,   // ieqe <reg1> <reg2> <u32_program_counter>
    IncrementReg,      // icrr <reg> - adds one to the register
}

/// Generate Instruction
fn incode_instr(input: [u8; 8]) -> u64 {
    u64::from_le_bytes(input)
}

fn deserialize_instruction(val: u8) -> Result<Instruction> {
    let out: Option<Instruction> = num::FromPrimitive::from_u8(val);
    if let Some(x) = out {
        Ok(x)
    } else {
        Err(anyhow!(
            "[DEATH]: FAILED TO DESERIALIZE INSTRUCTION, {} PROVIDED",
            val
        ))
    }
}

fn deserialize_u32_array(idx: usize, ray: &[u8; 8]) -> u32 {
    u32::from_le_bytes([ray[idx], ray[idx + 1], ray[idx + 2], ray[idx + 3]])
}

struct MemoryController<'b, const N: usize> {
    memory: &'b mut Memory<N>,
}

impl<'b, const N: usize> MemoryController<'b, N> {
    fn new_from(input: &'b mut Memory<N>) -> Self {
        Self { memory: input }
    }

    fn load_program_external(&mut self, ext_prg: &Vec<u64>, idx: usize) {
        for i in 0..ext_prg.len() {
            self.write((i + idx) as u32, ext_prg[i]);
        }
    }

    fn read(&self, idx: u32) -> u64 {
        self.memory.data[idx as usize].clone()
    }

    fn write(&mut self, idx: u32, val: u64) {
        self.memory.data[idx as usize] = val;
    }
}

struct Memory<const N: usize> {
    data: [u64; N], // we assume we have u32 worth of memory
}

impl<const N: usize> Memory<N> {
    fn new() -> Self {
        Self { data: [0_u64; N] }
    }
}

struct CPU<'a, const N: usize> {
    memory_controller: &'a mut MemoryController<'a, N>,
    reg_array: [u64; 8],
    current_instruction: [u8; 8],
    program_counter: u32,
}

impl<'a, const N: usize> CPU<'a, N> {
    fn print_state(&self) {
        println!("Registers: {:?}", &self.reg_array);
        println!("Current I: {:?}", &self.current_instruction);
        println!("Program C: {}", &self.program_counter);
        println!("Memory  H: {:?}", &self.mem_header());
    }

    fn mem_header(&self) -> String {
        format!("{:?}", &self.memory_controller.memory.data[0..30])
    }

    fn new(mc: &'a mut MemoryController<'a, N>) -> Self {
        Self {
            memory_controller: mc,
            reg_array: [0_u64; 8],
            current_instruction: [0_u8; 8],
            program_counter: 0_u32,
        }
    }

    fn run(&mut self) {
        let mut last = true;

        while last {
            last = self.cycle();
        }
    }

    fn run_debug(&mut self) {
        let mut last = true;

        while last {
            last = self.cycle_debug();
        }
    }

    fn cycle(&mut self) -> bool {
        // load
        self.load_instruction();

        let tmp_pc = self.program_counter.clone();

        // execute
        let out = self.execute();

        if tmp_pc == self.program_counter {
            self.incr();
        } // else we manipulated the program counter in execute() so we don't want to mess with it here.

        out
    }

    fn cycle_debug(&mut self) -> bool {
        // load
        self.load_instruction();

        println!("{:?}", &self.reg_array);
        println!(
            "PC: {}\nCI: {:?}\n",
            self.program_counter,
            deserialize_instruction(self.current_instruction[0])
        );

        let tmp_pc = self.program_counter.clone();

        // execute
        let out = self.execute();

        if tmp_pc == self.program_counter {
            self.incr();
        } // else we manipulated the program counter in execute() so we don't want to mess with it here.

        out
    }

    fn load_instruction(&mut self) {
        self.current_instruction = self
            .memory_controller
            .read(self.program_counter)
            .to_le_bytes();
    }

    fn read_from_reg(&self, idx: u8) -> u64 {
        self.reg_array[idx as usize]
    }

    fn write_to_reg(&mut self, idx: u8, val: u64) {
        self.reg_array[idx as usize] = val;
    }

    fn write_to_program_counter(&mut self, val: u32) {
        self.program_counter = val;
    }

    fn execute(&mut self) -> bool {
        /// this is gonna be the biggie

        let out: bool = match deserialize_instruction(self.current_instruction[0]) {
            Ok(instr) => match instr {
                /// ext - the computer does nothing, it just dies
                Instruction::Exit => {
                    return false;
                }
                Instruction::LoadFromMem => {
                    /// lod <mem_address> <register>
                    let mem_addr: u32 = deserialize_u32_array(1, &self.current_instruction);
                    let reg_addr: u8 = self.current_instruction[5];
                    let val: u64 = self.memory_controller.read(mem_addr);
                    self.write_to_reg(reg_addr, val);
                    return true;
                }
                /// wrt <register> <mem_address>
                Instruction::WriteToMem => {
                    let reg_addr: u8 = self.current_instruction[1];
                    let mem_addr: u32 = deserialize_u32_array(2, &self.current_instruction);
                    let out = self.read_from_reg(reg_addr);
                    self.memory_controller.write(mem_addr, out);
                    return true;
                }
                /// add <reg1> <reg2> <reg3> - adds reg1 to reg2 and writes to reg3
                Instruction::Add => {
                    let read_1_addr: u8 = self.current_instruction[1];
                    let read_2_addr: u8 = self.current_instruction[2];
                    let write_addr: u8 = self.current_instruction[3];

                    let out = self.read_from_reg(read_1_addr) + self.read_from_reg(read_2_addr);
                    self.write_to_reg(write_addr, out);

                    return true;
                }
                /// sub <reg1> <reg2> <reg3> - subs reg1 to reg2 and writes to reg3
                Instruction::Sub => {
                    let read_1_addr: u8 = self.current_instruction[1];
                    let read_2_addr: u8 = self.current_instruction[2];
                    let write_addr: u8 = self.current_instruction[3];

                    let out = self.read_from_reg(read_1_addr) + self.read_from_reg(read_2_addr);
                    self.write_to_reg(write_addr, out);

                    return true;
                }
                /// spc <u32_value> - sets the program counter to the u32 in the instruction
                Instruction::SetProgramCounter => {
                    self.write_to_program_counter(deserialize_u32_array(
                        1,
                        &self.current_instruction,
                    ));
                    return true;
                }
                /// clra - sets every register to zero
                Instruction::ClearAllRegisters => {
                    for idx in 0..8 {
                        self.write_to_reg(idx, 0);
                    }
                    return true;
                }
                /// clr <reg_addr> - sets this register to zero
                Instruction::ClearRegister => {
                    self.write_to_reg(self.current_instruction[1], 0);
                    return true;
                }
                /// rw <reg1> <reg2> - writes the value of register 1 to register 2
                Instruction::RegisterWrite => {
                    let idxleft = self.current_instruction[1];
                    let idxright = self.current_instruction[2];
                    self.write_to_reg(idxright, self.read_from_reg(idxleft));
                    return true;
                }
                /// ieqe <reg1> <reg2> <u32_program_counter>
                Instruction::IfEqSPCElsePass => {
                    let reg1 = self.current_instruction[1];
                    let reg2 = self.current_instruction[2];
                    if self.read_from_reg(reg1) == self.read_from_reg(reg2) {
                        let pcu32 = deserialize_u32_array(3, &self.current_instruction);
                        self.program_counter = pcu32;
                    }
                    return true;
                }
                /// icrr <reg> - adds one to the register
                Instruction::IncrementReg => {
                    let reg = self.current_instruction[1];
                    let out = self.read_from_reg(reg) + 1;
                    self.write_to_reg(reg, out);
                    return true;
                }
            },
            Err(x) => {
                panic!("{}", x);
            }
        };
        out
    }

    fn incr(&mut self) {
        self.program_counter += 1;
    }
}

fn fib_n(n: usize) -> u64 {
    let mut a = 0_u64;
    let mut b = 1_u64;

    for _ in 0..n {
        let tmp = b + a;
        a = b;
        b = tmp;
    }

    return b;
}

// we want everything to be little endian

fn comp_fib(n: usize) {
    let mut memory = Memory::<100>::new();

    /*
    [
        0, // val
        1, // val
        100, // val
        [1, 0,0,0,0, 0, 0,0], // load 1
        [1, 1,0,0,0, 1, 0,0], // load 2
        [1, 2,0,0,0, 3, 0,0], // load 3
        [9, 3, 4, 0,0,0,0, 0] // if reg4 == reg 3, set to mem0
        [3, 0, 1, 7, 0,0,0],  // add
        [8, 1, 0, 0,0,0,0,0] // reg write
        [8, 7, 1, 0,0,0,0,0] // reg write
        [10, 4, 0,0,0,0,0,0] // increment register 4
        [5, 6,0,0,0, 0,0,0] // spc 6

    ]
    */

    let program: Vec<u64> = vec![
        0,                                       // val
        1,                                       // val
        n as u64,                                      // val
        incode_instr([1, 0, 0, 0, 0, 0, 0, 0]),  // load 1
        incode_instr([1, 1, 0, 0, 0, 1, 0, 0]),  // load 2
        incode_instr([1, 2, 0, 0, 0, 2, 0, 0]),  // load 3
        incode_instr([9, 2, 3, 0, 0, 0, 0, 0]),  // ifelsepass
        incode_instr([3, 0, 1, 7, 0, 0, 0, 0]),  // add
        incode_instr([8, 1, 0, 0, 0, 0, 0, 0]),  // reg write
        incode_instr([8, 7, 1, 0, 0, 0, 0, 0]),  // reg write
        incode_instr([10, 3, 0, 0, 0, 0, 0, 0]), // increment register 4
        incode_instr([5, 6, 0, 0, 0, 0, 0, 0]),  // spc 6
    ];

    let mut memory_controller = MemoryController::new_from(&mut memory);

    memory_controller.load_program_external(&program, 0);

    let mut computer = CPU::new(&mut memory_controller);
    computer.program_counter = 3;
    computer.run();
}

fn main() {
    let mut memory = Memory::<100>::new();
    let n = 75;
    /*
    [
        0, // val
        1, // val
        100, // val
        [1, 0,0,0,0, 0, 0,0], // load 1
        [1, 1,0,0,0, 1, 0,0], // load 2
        [1, 2,0,0,0, 3, 0,0], // load 3
        [9, 3, 4, 0,0,0,0, 0] // if reg4 == reg 3, set to mem0
        [3, 0, 1, 7, 0,0,0],  // add
        [8, 1, 0, 0,0,0,0,0] // reg write
        [8, 7, 1, 0,0,0,0,0] // reg write
        [10, 4, 0,0,0,0,0,0] // increment register 4
        [5, 6,0,0,0, 0,0,0] // spc 6

    ]
    */

    let program: Vec<u64> = vec![
        0,                                       // val
        1,                                       // val
        n as u64,                                      // val
        incode_instr([1, 0, 0, 0, 0, 0, 0, 0]),  // load 1
        incode_instr([1, 1, 0, 0, 0, 1, 0, 0]),  // load 2
        incode_instr([1, 2, 0, 0, 0, 2, 0, 0]),  // load 3
        incode_instr([3, 0, 1, 7, 0, 0, 0, 0]),  // add
        incode_instr([8, 1, 0, 0, 0, 0, 0, 0]),  // reg write
        incode_instr([8, 7, 1, 0, 0, 0, 0, 0]),  // reg write
        incode_instr([10, 3, 0, 0, 0, 0, 0, 0]), // increment register 4
        incode_instr([9, 2, 3, 12, 0, 0, 0, 0]),  // ifelsepass
        incode_instr([5, 6, 0, 0, 0, 0, 0, 0]),  // spc 6
        incode_instr([2,7,20,0,0,0,0,0])         // write from reg to mem
    ];

    let mut memory_controller = MemoryController::new_from(&mut memory);

    memory_controller.load_program_external(&program, 0);

    let mut computer = CPU::new(&mut memory_controller);
    computer.program_counter = 3;
    computer.run_debug();
    computer.print_state();
}
