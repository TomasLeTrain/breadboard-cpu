use opcode_gen::output::Output;

struct Alu {}

impl Alu {
    fn new() -> Self {
        Self {}
    }

    fn update(&self, opcode_addr: u32, a: u8, b: u8) {
        // TODO: impl
    }

    fn get_flags(&self) -> Option<u8> {
        // TODO: impl
        None
    }

    fn bout(&self) -> Option<u8> {
        // TODO: impl
        None
    }
}

struct AddressRegister {
    state: Option<u16>,
}

impl AddressRegister {
    fn new() -> Self {
        Self { state: None }
    }

    fn reset(&mut self) {
        self.state = Some(0);
    }

    fn load_high(&mut self, val: u8) {
        let new_val = (self.state.unwrap() & 0xff) | ((val as u16) << 8);
        self.state = Some(new_val);
    }

    fn load_low(&mut self, val: u8) {
        let new_val = (self.state.unwrap() & !0xff) | ((val as u16) << 8);
        self.state = Some(new_val);
    }

    fn increment(&mut self) {
        if let Some(e) = &mut self.state {
            *e += 1
        }
    }

    fn decrement(&mut self) {
        if let Some(e) = &mut self.state {
            *e -= 1
        }
    }

    fn aout(&self) -> Option<u16> {
        self.state
    }
}

trait DataRegister {
    fn state_as_mut(&mut self) -> &mut Option<u8>;
    fn state(&self) -> Option<u8>;

    fn load(&mut self, val: u8) {
        *self.state_as_mut() = Some(val);
    }

    fn bout(&self) -> Option<u8> {
        self.state()
    }

    fn reset(&mut self) {
        *self.state_as_mut() = Some(0);
    }
}

struct Register {
    state: Option<u8>,
}

impl Register {
    fn new() -> Self {
        Self { state: None }
    }
}

impl DataRegister for Register {
    fn state_as_mut(&mut self) -> &mut Option<u8> {
        &mut self.state
    }

    fn state(&self) -> Option<u8> {
        self.state
    }
}

struct ShiftRegister {
    state: Option<u8>,
}

impl DataRegister for ShiftRegister {
    fn state_as_mut(&mut self) -> &mut Option<u8> {
        &mut self.state
    }

    fn state(&self) -> Option<u8> {
        self.state
    }
}

impl ShiftRegister {
    fn new() -> Self {
        Self { state: None }
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
}

impl DataRegister for CountRegister {
    fn state_as_mut(&mut self) -> &mut Option<u8> {
        &mut self.state
    }

    fn state(&self) -> Option<u8> {
        self.state
    }
}

impl CountRegister {
    fn new() -> Self {
        Self { state: None }
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

    opcode_rom1: Rom,
    opcode_rom2: Rom,

    opcode_latch1: Register,
    opcode_latch2: Register,

    data_rom: Rom,
    data_ram: Ram,

    alu: Alu,

    fast_clk: bool,
    clk: bool,

    halt: bool,

    bus_val: Option<u8>,
    addr_val: Option<u16>,
}

impl CpuState {
    fn new() -> Self {
        Self {
            a: Register::new(),
            b: Register::new(),
            x: ShiftRegister::new(),
            y: ShiftRegister::new(),
            z: Register::new(),
            flags: Register::new(),

            mar: AddressRegister::new(),
            pc: AddressRegister::new(),
            sp: AddressRegister::new(),

            ir: Register::new(),
            ir2: Register::new(),

            step_counter: CountRegister::new(),
            opcode_rom1: Rom::new(1 << 17),
            opcode_rom2: Rom::new(1 << 17),

            opcode_latch1: Register::new(),
            opcode_latch2: Register::new(),

            data_rom: Rom::new(1 << 15),
            data_ram: Ram::new(1 << 15),

            alu: Alu::new(),

            fast_clk: true,
            clk: true,

            halt: false,

            bus_val: None,
            addr_val: None,
        }
    }

    fn mem_read(&self) -> Option<u8> {
        let addr = self.addr_val.unwrap();
        let masked_addr = addr & !(1 << 15);
        if addr & 1 << 15 != 0 {
            // ram
            self.data_ram.read(masked_addr)
        } else {
            // rom
            self.data_rom.read(masked_addr as u32)
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
        0
    }

    fn get_opcode_output(&self) -> Output {
        let data = (self.opcode_latch1.bout().unwrap() as u16)
            | ((self.opcode_latch2.bout().unwrap() as u16) << 8);
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
        self.flags.bout().unwrap() & flag_select != 0
    }

    fn perform_mutable_actions(&mut self) {
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

    // simulates a clock cycle (higher frequency) happening
    fn step_half_clk(&mut self) {
        if self.halt {
            return;
        }

        self.fast_clk = !self.fast_clk;
        // step on fast_clk low to high
        if self.fast_clk {
            self.clk = !self.clk;
        }

        if self.clk {
            // low->high clock right now
            // perform whatever op
            self.perform_mutable_actions();
        } else {
            // high->low clock right now
            // clock in rom
            let opcode_addr = self.get_opcode_addr();

            self.opcode_latch1
                .load(self.opcode_rom1.read(opcode_addr).unwrap());
            self.opcode_latch2
                .load(self.opcode_rom2.read(opcode_addr).unwrap());

            let control_actions = self.get_opcode_output();

            // increment pc cnt here
            if control_actions.get_pc_cnt() {
                self.pc.increment();
            }

            // update outputs of all non-clocked actions (output to buses)
            self.bus_val = self.bout_value();
            self.addr_val = self.addr_value();
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

        self.opcode_latch1.reset();
        self.opcode_latch2.reset();

        self.fast_clk = true;
        self.clk = true;

        self.bus_val = None;
        self.addr_val = None;

        self.halt = false;
    }
}

fn main() {
    let mut state = CpuState::new();

    state.reset();

    // TODO: load opcode roms from bin files

    for i in 1..=10 {
        state.step_half_clk();
    }
}
