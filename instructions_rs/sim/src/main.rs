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

    fn load(&mut self, val: u16) {
        self.state = Some(val);
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

    fast_clk: bool,
    clk: bool,

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

            fast_clk: true,
            clk: true,

            bus_val: None,
            addr_val: None,
        }
    }

    fn get_opcode_addr(&self) -> u32 {
        0
    }

    fn get_opcode_data(&self) -> u16 {
        (self.opcode_latch1.bout().unwrap() as u16)
            | ((self.opcode_latch2.bout().unwrap() as u16) << 8)
    }

    fn perform_op(&mut self) {
        let mut bus_val: Option<u8> = None;
        let mut addr_val: Option<u16> = None;
        // at this point the bus and addr should be already updated from the last half clock phase
    }

    // simulates a clock cycle (higher frequency) happening
    fn step_half_clk(&mut self) {
        self.fast_clk = !self.fast_clk;
        // step on fast_clk low to high
        if self.fast_clk {
            self.clk = !self.clk;
        }

        if self.clk {
            // low->high clock right now
            // perform whatever op
            self.perform_op();
        } else {
            // high->low clock right now
            // clock in rom
            let opcode_addr = self.get_opcode_addr();

            self.opcode_latch1
                .load(self.opcode_rom1.read(opcode_addr).unwrap());
            self.opcode_latch2
                .load(self.opcode_rom2.read(opcode_addr).unwrap());

            // increment pc cnt here
            if self.get_opcode_data() & (1 << 15) != 0 {
                self.pc.increment();
            }

            // update outputs of all non-clocked actions (output to buses)
        }
    }
}

fn main() {
    let mut state = CpuState::new();

    // TODO: load opcode roms from bin files

    for i in 1..=10 {
        state.step_half_clk();
    }
}
