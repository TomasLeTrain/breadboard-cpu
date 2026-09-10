// cpu state in a

struct AddressRegister {
    state: u16,
}

impl AddressRegister {
    fn reset(&mut self) {
        self.state = 0;
    }

    fn load(&mut self, val: u16) {
        self.state = val;
    }

    fn increment(&mut self) {
        self.state += 1
    }
    fn decrement(&mut self) {
        self.state -= 1
    }

    fn aout(&self) -> u16 {
        self.state
    }
}

trait DataRegister {
    fn state_as_mut(&mut self) -> &mut u8;
    fn state(&self) -> u8;

    fn load(&mut self, val: u8) {
        *self.state_as_mut() = val;
    }

    fn bout(&self) -> u8 {
        self.state()
    }
}

struct Register {
    state: u8,
}

impl DataRegister for Register {
    fn state_as_mut(&mut self) -> &mut u8 {
        &mut self.state
    }

    fn state(&self) -> u8 {
        self.state
    }
}

struct ShiftRegister {
    state: u8,
}

impl DataRegister for ShiftRegister {
    fn state_as_mut(&mut self) -> &mut u8 {
        &mut self.state
    }

    fn state(&self) -> u8 {
        self.state
    }
}

impl ShiftRegister {
    fn shift_left(&mut self) {
        self.state <<= 1;
    }

    fn shift_right(&mut self) {
        self.state >>= 1;
    }
}

struct CountRegister {
    state: u8,
}

impl DataRegister for CountRegister {
    fn state_as_mut(&mut self) -> &mut u8 {
        &mut self.state
    }

    fn state(&self) -> u8 {
        self.state
    }
}

impl CountRegister {
    fn increment(&mut self) {
        self.state += 1;
    }
}

struct CpuState {
    a: Register,
    b: Register,
    x: ShiftRegister,
    y: ShiftRegister,
    z: Register,

    mar: AddressRegister,
    pc: AddressRegister,
    sp: AddressRegister,

    ir: Register,
    ir2: Register,

    step_counter: CountRegister,
}

fn main() {
    println!("Hello, world!");
}
