use std::{
    fs::{self, File},
    io::Read,
};

use opcode_gen::{
    diagnostic_from_addr, get_instruction_set,
    instructions::IstrSet,
    opcode::{self, Opcode},
    output::Output,
};

enum AluOp {
    Addition, // carry decided independently
    Subtract, // carry decided independently
    And,
    Or,
    Xor,
    Not,
    Compare,
}

impl AluOp {
    fn from_opcode_addr(addr: u32) -> Self {
        let opcode = Opcode::from_addr(addr);
        // higher bits of ir
        let high_bits = (opcode.ir >> 4) & 0b111;

        match high_bits {
            0 => Self::Subtract,
            1 => Self::Compare,
            2 => Self::Addition,
            3 => Self::Addition,
            4 => Self::Not,
            5 => Self::Xor,
            6 => Self::Or,
            7 => Self::And,
            _ => unreachable!(),
        }
    }
}

struct Alu {
    a: Option<u8>,
    b: Option<u8>,
    result: Option<u8>,
    flags: Option<u8>,
}

impl Alu {
    fn new() -> Self {
        Self {
            a: None,
            b: None,
            result: None,
            flags: None,
        }
    }

    fn update(
        &mut self,
        opcode_addr: u32,
        flag_select: u8,
        flags: Option<u8>,
        a: Option<u8>,
        b: Option<u8>,
    ) {
        self.a = a;
        self.b = b;
        let op = AluOp::from_opcode_addr(opcode_addr);

        let using_carry = flag_select != 0;
        let carry_on = flags.unwrap() & (1 << flag_select) != 0;

        if let Some(a) = self.a
            && let Some(b) = self.b
        {
            self.result = Some(match op {
                AluOp::Addition => {
                    if using_carry {
                        a.wrapping_add(b).wrapping_add(if carry_on { 1 } else { 0 })
                    } else {
                        a.wrapping_add(b)
                    }
                }
                AluOp::Subtract => {
                    if using_carry {
                        // TODO: math might be wrong?
                        // carry is inverted for subtraction
                        a.wrapping_add(!b)
                            .wrapping_add(if carry_on { 0 } else { 1 })
                    } else {
                        a.wrapping_add(!b).wrapping_add(1)
                    }
                }
                AluOp::And => a & b,
                AluOp::Or => a | b,
                AluOp::Xor => a ^ b,
                AluOp::Not => !a,
                // sub mode, no carry
                AluOp::Compare => a.wrapping_add(!b),
            });

            // TODO: impl
            self.flags = Some(0);
        } else {
            self.result = None;
            self.flags = None;
        }
    }

    fn get_flags(&self) -> Option<u8> {
        self.flags
    }

    fn bout(&self) -> Option<u8> {
        self.result
    }
}

struct AddressRegister {
    state: Option<u16>,
    name: String,
}

impl AddressRegister {
    fn new(name: &str) -> Self {
        Self {
            state: None,
            name: name.to_string(),
        }
    }

    fn reset(&mut self) {
        self.state = Some(0);
    }

    fn load_high(&mut self, val: u8) {
        println!("load {val:x} into high of {}", self.name);
        let new_val = (self.state.unwrap() & 0xff) | ((val as u16) << 8);
        self.state = Some(new_val);
    }

    fn load_low(&mut self, val: u8) {
        println!("load {val:x} into low of {}", self.name);
        let new_val = (self.state.unwrap() & !0xff) | (val as u16);
        self.state = Some(new_val);
    }

    fn increment(&mut self) {
        println!("increment {}", self.name);
        if let Some(e) = &mut self.state {
            *e += 1
        }
    }

    fn decrement(&mut self) {
        println!("decrement {}", self.name);
        if let Some(e) = &mut self.state {
            *e -= 1
        }
    }

    fn aout(&self) -> Option<u16> {
        println!("aout value {:x?} from {}", self.state, self.name);
        self.state
    }
}

trait DataRegister {
    fn state_as_mut(&mut self) -> &mut Option<u8>;
    fn state(&self) -> Option<u8>;
    fn name(&self) -> &str;

    fn load(&mut self, val: u8) {
        println!("load {val:x} into {}", self.name());
        *self.state_as_mut() = Some(val);
    }

    fn bout(&self) -> Option<u8> {
        println!("bout {:x?} from {}", self.state(), self.name());
        self.state()
    }

    fn reset(&mut self) {
        println!("reset {}", self.name());
        *self.state_as_mut() = Some(0);
    }
}

struct Register {
    state: Option<u8>,
    name: String,
}

impl Register {
    fn new(name: &str) -> Self {
        Self {
            state: None,
            name: name.to_string(),
        }
    }
}

impl DataRegister for Register {
    fn state_as_mut(&mut self) -> &mut Option<u8> {
        &mut self.state
    }

    fn state(&self) -> Option<u8> {
        self.state
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}

struct ShiftRegister {
    state: Option<u8>,
    name: String,
}

impl DataRegister for ShiftRegister {
    fn state_as_mut(&mut self) -> &mut Option<u8> {
        &mut self.state
    }

    fn state(&self) -> Option<u8> {
        self.state
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl ShiftRegister {
    fn new(name: &str) -> Self {
        Self {
            state: None,
            name: name.to_string(),
        }
    }

    fn shift_left(&mut self) {
        if let Some(e) = &mut self.state {
            *e <<= 1;
        }
    }

    fn shift_right(&mut self) {
        if let Some(e) = &mut self.state {
            *e >>= 1;
        }
    }
}

struct CountRegister {
    state: Option<u8>,
    name: String,
}

impl DataRegister for CountRegister {
    fn state_as_mut(&mut self) -> &mut Option<u8> {
        &mut self.state
    }

    fn state(&self) -> Option<u8> {
        self.state
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl CountRegister {
    fn new(name: &str) -> Self {
        Self {
            state: None,
            name: name.to_string(),
        }
    }

    fn increment(&mut self) {
        if let Some(e) = &mut self.state {
            *e += 1
        }
    }
}

struct Ram {
    state: Vec<Option<u8>>,
}

impl Ram {
    fn new(len: usize) -> Self {
        Self {
            state: vec![None; len],
        }
    }

    fn read(&self, addr: u16) -> Option<u8> {
        self.state[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        println!("writing {val:x} at {addr:x} to ram");
        self.state[addr as usize] = Some(val);
    }
}

struct Rom {
    state: Vec<Option<u8>>,
}
impl Rom {
    fn new(len: usize) -> Self {
        Self {
            state: vec![None; len],
        }
    }

    fn read(&self, addr: u32) -> Option<u8> {
        self.state[addr as usize]
    }

    fn load_image(&mut self, image: Vec<u8>) {
        assert_eq!(self.state.len(), image.len());
        self.state = image.into_iter().map(Some).collect()
    }
}

struct CpuState {
    a: Register,
    b: Register,
    x: ShiftRegister,
    y: ShiftRegister,
    z: Register,

    flags: Register,

    mar: AddressRegister,
    pc: AddressRegister,
    sp: AddressRegister,

    ir: Register,
    ir2: Register,

    step_counter: CountRegister,

    opcode_rom0: Rom,
    opcode_rom1: Rom,

    opcode_latch0: Register,
    opcode_latch1: Register,

    data_rom: Rom,
    data_ram: Ram,

    alu: Alu,

    clk: bool,

    halt: bool,

    bus_val: Option<u8>,
    addr_val: Option<u16>,

    // used for debugging
    istr_set: IstrSet,
}

impl CpuState {
    fn new() -> Self {
        Self {
            a: Register::new("A"),
            b: Register::new("B"),
            x: ShiftRegister::new("X"),
            y: ShiftRegister::new("Y"),
            z: Register::new("Z"),
            flags: Register::new("FLAGS"),

            mar: AddressRegister::new("MAR"),
            pc: AddressRegister::new("PC"),
            sp: AddressRegister::new("SP"),

            ir: Register::new("IR"),
            ir2: Register::new("IR2"),

            step_counter: CountRegister::new("STEP"),
            opcode_rom0: Rom::new(1 << 17),
            opcode_rom1: Rom::new(1 << 17),

            opcode_latch0: Register::new("OPCODE_LATCH1"),
            opcode_latch1: Register::new("OPCODE_LATCH2"),

            data_rom: Rom::new(1 << 17),
            data_ram: Ram::new(1 << 15),

            alu: Alu::new(),

            clk: true,

            halt: false,

            bus_val: None,
            addr_val: None,
            istr_set: get_instruction_set().1,
        }
    }

    fn mem_read(&self) -> Option<u8> {
        let addr = self.addr_val.unwrap();

        let masked_addr = addr & !(1 << 15);
        if addr & 1 << 15 != 0 {
            // ram
            let data = self.data_ram.read(masked_addr);
            println!("ram read {data:x?} from {masked_addr:x}");
            data
        } else {
            // rom
            let data = self.data_rom.read(masked_addr as u32);
            println!("rom read: {data:x?} from {masked_addr:x}");
            data
        }
    }

    fn mem_write(&mut self) {
        let addr = self.addr_val.unwrap();
        let val = self.bus_val.unwrap();
        let masked_addr = addr & !(1 << 15);
        if addr & 1 << 15 != 0 {
            // ram
            self.data_ram.write(masked_addr, val);
        } else {
            // rom
            // nop
            // TODO: alert of nop
            panic!("writing to rom!")
        }
    }

    fn bout_value(&self) -> Option<u8> {
        let bout_val = match self.get_opcode_output().get_bout() {
            0 => None, // unused
            1 => self.mem_read(),
            2 => {
                // lo
                Some((self.addr_value().unwrap() & 0xff) as u8)
            }
            3 => {
                // hi
                Some(((self.addr_value().unwrap() & (0xff << 8)) >> 8) as u8)
            }
            4 => None, // MarCnt
            5 => None, // Halt
            6 => None, // SpInc
            7 => None, // SpDec
            8 => self.a.bout(),
            9 => self.b.bout(),
            10 => todo!(), // TODO: add keyb
            11 => self.alu.bout(),
            12 => self.flags.bout(),
            13 => self.x.bout(),
            14 => self.y.bout(),
            15 => self.z.bout(),
            _ => unreachable!(),
        };

        if self.get_opcode_output().get_misc() == 3
            && self.get_opcode_output().get_flag_select() == 5
        {
            assert_eq!(bout_val, None);

            // TODO: VramRead
            todo!()
        } else {
            bout_val
        }
    }

    fn addr_value(&self) -> Option<u16> {
        match self.get_opcode_output().get_addr() {
            0 => None,
            1 => self.pc.aout(),
            2 => self.mar.aout(),
            3 => self.sp.aout(),
            _ => unreachable!(),
        }
    }

    fn get_opcode_addr(&self) -> u32 {
        Opcode {
            step: self.step_counter.state().unwrap(),
            ir: self.ir.state().unwrap(),
            ir2: self.ir2.state().unwrap(),
            not_vram_active: false,
        }
        .to_addr()
    }

    fn get_opcode_output(&self) -> Output {
        let data = (self.opcode_latch0.state().unwrap() as u16)
            | ((self.opcode_latch1.state().unwrap() as u16) << 8);
        Output::from_output_data(data)
    }

    fn pc_jump_enabled(&self) -> bool {
        let control_actions = self.get_opcode_output();
        let flag_select = control_actions.get_flag_select();

        // 0 is direct jump
        if flag_select == 0 {
            return true;
        }

        // otherwise its conditional jump
        self.flags.state().unwrap() & flag_select != 0
    }

    fn perform_mutable_actions(&mut self) {
        // increase step
        // NOTE: must be at the top, since it might get reset later
        self.step_counter.increment();

        // at this point the bus and addr should be already updated from the last half clock phase
        let control_actions = self.get_opcode_output();

        match control_actions.get_write() {
            0 => (),
            1 => self.ir2.load(self.bus_val.unwrap()),
            2 => self.sp.load_high(self.bus_val.unwrap()),
            3 => self.sp.load_low(self.bus_val.unwrap()),
            4 => self.mar.load_high(self.bus_val.unwrap()),
            5 => self.mar.load_low(self.bus_val.unwrap()),
            6 => self.mem_write(),
            7 => self.z.load(self.bus_val.unwrap()),
            8 => self.a.load(self.bus_val.unwrap()),
            9 => self.b.load(self.bus_val.unwrap()),
            10 => {
                if self.pc_jump_enabled() {
                    self.pc.load_high(self.bus_val.unwrap())
                }
            }
            11 => {
                if self.pc_jump_enabled() {
                    self.pc.load_low(self.bus_val.unwrap())
                }
            }
            12 => unreachable!(),
            13 => self.x.load(self.bus_val.unwrap()),
            14 => self.y.load(self.bus_val.unwrap()),
            15 => self.ir.load(self.bus_val.unwrap()),
            _ => unreachable!(),
        };

        let other = control_actions.get_misc();

        // other mux enabled
        if other == 3 {
            match control_actions.get_flag_select() {
                0 => self.x.shift_left(),
                1 => self.x.shift_right(),
                2 => self.y.shift_left(),
                3 => self.y.shift_right(),
                4 => (), // TODO: VramWrite
                _ => unreachable!(),
            }
        } else {
            match other {
                0 => (),
                1 => self.flags.load(self.alu.get_flags().unwrap()), //FlagWriteAlu
                2 => self.step_counter.reset(),                      // step reset
                _ => unreachable!(),
            };
        }

        match self.get_opcode_output().get_bout() {
            0..=3 => (),
            4 => self.mar.increment(), // MarCnt
            5 => self.halt = true,     // Halt
            6 => self.sp.increment(),  // SpInc
            7 => self.sp.decrement(),  // SpDec
            8..=15 => (),
            _ => unreachable!(),
        };
    }

    fn is_halt(&self) -> bool {
        self.halt
    }

    // simulates a clock cycle (higher frequency) happening
    fn step_half_clk(&mut self) {
        if self.halt {
            return;
        }

        self.clk = !self.clk;

        if self.clk {
            // low->high clock right now
            // perform whatever op
            println!("low to high clk");
            self.perform_mutable_actions();
        } else {
            // high->low clock right now
            // clock in rom
            println!("high to low clk");

            let opcode_addr = self.get_opcode_addr();
            println!("opcode_addr: {opcode_addr:x}");

            diagnostic_from_addr(opcode_addr, &self.istr_set);

            self.opcode_latch0
                .load(self.opcode_rom0.read(opcode_addr).unwrap());
            self.opcode_latch1
                .load(self.opcode_rom1.read(opcode_addr).unwrap());

            // println!("latched opcode stuff");

            let control_actions = self.get_opcode_output();
            // println!("control actions {:?}", control_actions.get_printable_data());

            // increment pc cnt here
            // NOTE: must happen before addr, in hardware it gets done on transition and read only
            // matters on low to high transition
            if control_actions.get_pc_cnt() {
                self.pc.increment();
            }

            // update outputs of all non-clocked actions (output to buses)
            // NOTE; must update addr first since bus depends on it
            self.addr_val = self.addr_value();
            self.bus_val = self.bout_value();

            self.alu.update(
                opcode_addr,
                control_actions.get_flag_select(),
                self.flags.state(),
                self.a.state(),
                self.b.state(),
            );
        }
    }

    fn reset(&mut self) {
        self.a.reset();
        self.b.reset();
        self.x.reset();
        self.y.reset();
        self.z.reset();
        self.flags.reset();

        self.mar.reset();
        self.pc.reset();
        self.sp.reset();

        self.ir.reset();
        self.ir2.reset();

        self.step_counter.reset();

        self.opcode_latch0.reset();
        self.opcode_latch1.reset();

        self.clk = true;

        self.bus_val = None;
        self.addr_val = None;

        self.halt = false;
    }

    fn load_opcode_roms(&mut self, rom0_path: &str, rom1_path: &str) {
        let mut rom0 = Vec::new();
        let mut rom1 = Vec::new();

        File::open(rom0_path)
            .unwrap()
            .read_to_end(&mut rom0)
            .unwrap();
        File::open(rom1_path)
            .unwrap()
            .read_to_end(&mut rom1)
            .unwrap();

        self.opcode_rom0.load_image(rom0);
        self.opcode_rom1.load_image(rom1);
    }

    fn load_program_rom(&mut self, rom_path: &str) {
        let mut rom = Vec::new();

        File::open(rom_path).unwrap().read_to_end(&mut rom).unwrap();

        self.data_rom.load_image(rom);
    }
}

fn main() {
    let mut state = CpuState::new();

    state.load_opcode_roms("../opcode_gen/rom0.bin", "../opcode_gen/rom1.bin");
    state.load_program_rom("../assembler/asm_bin.bin");

    state.reset();

    let mut i = 1;
    loop {
        println!();
        println!("doing step {i}");
        state.step_half_clk();
        if state.is_halt() {
            break;
        }
        i += 1;
    }

    println!();
    println!("halted!");

    // print state of cpu
    println!("a: {:?}", state.a.state());
    println!("b: {:?}", state.b.state());
    println!("x: {:?}", state.x.state());
    println!("y: {:?}", state.y.state());
    println!("z: {:?}", state.z.state());

    println!("pc: 0x{:x?}", state.pc.aout().unwrap());
    println!("mar: 0x{:x?}", state.mar.aout().unwrap());
    println!("sp: 0x{:x?}", state.sp.aout().unwrap());
}
