use rand::Rng;

type Stack<T> = Vec<T>; 
pub enum Chip8EmulatorError {
    StackOverflow,
}
const STACK_CAPACITY: usize = 12;

pub fn add_instruction(registers: &mut [u8; 16], index: usize, value: u8)
{
    let (result, is_overflow) = (*registers.get(index).expect("NON VALID REGISTER")).overflowing_add(value);
    registers[0xF as usize] = is_overflow.into();
    registers[index] = result;
}

pub fn add_no_overflow_instruction(registers: &mut [u8; 16], index: usize, value: u8)
{
    let (result, _is_overflow) = (*registers.get(index).expect("NON VALID REGISTER")).overflowing_add(value);
    registers[index] = result;
}

pub fn sub_instruction(registers: &mut [u8; 16], index: usize, value: u8)
{
    let (result, is_borrow) = (*registers.get(index).expect("NON VALID REGISTER")).overflowing_sub(value);
    registers[0xF as usize] = (!is_borrow).into();
    registers[index] = result;
}

pub fn shl_instruction(registers: &mut [u8; 16], index: usize)
{
    registers[0xF as usize] = ((*registers.get(index).expect("NON VALID REGISTER")) & 0b1000_0000) >> 7; // get the msb in r1, shift it 7 places right to get 1 or 0
    registers[index] = (*registers.get(index).expect("NON VALID REGISTER")) << 1;
}

pub fn shr_instruction(registers: &mut [u8; 16], index: usize)
{
    registers[0xF as usize] = (*registers.get(index).expect("NON VALID REGISTER")) & 0b0000_0001; // get the lsb in r1 in a form of 1 or 0
    registers[index] = (*registers.get(index).expect("NON VALID REGISTER")) >> 1;
}

pub fn jump_instruction(pc: &mut usize, next_address: u16)
{
    *pc = next_address as usize;
}

pub fn call_instruction(pc: &mut usize, next_address: u16, stack: &mut Stack<u16>) -> Result<(), Chip8EmulatorError>
{
    if stack.len() >= STACK_CAPACITY {
        return Err(Chip8EmulatorError::StackOverflow);
    }

    stack.push(*pc as u16);
    *pc = next_address as usize;
    Ok(())
}

pub fn jump_v0_instruction(registers: [u8; 16], pc: &mut usize, next_address: u16)
{
    *pc = (registers[0] as u16 + next_address) as usize;
}

pub fn rand_instruction(registers: &mut [u8; 16], index: usize, value: u8)
{
    let mut random = rand::thread_rng();
    registers[index] = random.gen_range(0..=255) & value;
}