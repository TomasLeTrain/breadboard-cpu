; some common utils

fn jsr(addr: Address) {
	return {
		pusha return_addr
		jmp addr, MAR
		return_addr:
	}
}

fn high_byte(x: u16) {
	let high_byte: u8 = (x >> 8) & 0xff
	return high_byte
}
