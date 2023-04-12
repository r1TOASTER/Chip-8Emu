use crate::registers::{ VF, Register};
use rand::Rng;

type Stack<T> = Vec<T>; 
pub enum Chip8EmulatorError {
    StackOverflow,
}
const STACK_CAPACITY: usize = 12;

pub fn add_instruction<R1>(r1: &mut R1, value: u8, overflow: &mut VF)
    where
    R1: Register<u8>,
{
    let (result, is_overflow) = r1.read().overflowing_add(value);
    overflow.write(is_overflow.into());
    r1.write(result);
}

pub fn add_no_overflow_instruction<R1>(r1: &mut R1, value: u8)
    where
    R1: Register<u8>,
{
    let (result, _is_overflow) = r1.read().overflowing_add(value);
    r1.write(result);
}

pub fn sub_instruction<R1>(r1: &mut R1, value: u8, borrow: &mut VF)
    where
    R1: Register<u8>,
{
    let (result, is_borrow) = r1.read().overflowing_sub(value);
    borrow.write((!is_borrow).into());
    r1.write(result);
}

pub fn shl_instruction<R1>(r1: &mut R1, shifted: &mut VF)
    where
    R1: Register<u8>,
{
    shifted.write((r1.read() & 0b1000_0000) >> 7); // get the msb in r1, shift it 7 places right to get 1 or 0
    r1.write(r1.read() << 1);
}

pub fn shr_instruction<R1>(r1: &mut R1, shifted: &mut VF)
    where
    R1: Register<u8>,
{
    shifted.write(r1.read() & 0b0000_0001); // get the lsb in r1 in a form of 1 or 0
    r1.write(r1.read() >> 1);
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

pub fn jump_v0_instruction<R1>(r0: &R1, pc: &mut usize, next_address: u16)
    where 
    R1: Register<u8>,
{
    *pc = (r0.read() as u16 + next_address) as usize;
}

pub fn rand_instruction<R1>(r1: &mut R1, value: u8)
    where
    R1: Register<u8>,
{
    let mut random = rand::thread_rng();
    r1.write(random.gen_range(0..=255) & value);
}