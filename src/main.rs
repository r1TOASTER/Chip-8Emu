use core::panic;
use std::thread;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::{Read, Result, ErrorKind};
use std::time::Duration;
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
};

use registers::Register;
use timers::Timer;

use crate::registers::Chip8Registers;
use crate::opcodes::{
                    add_instruction, sub_instruction,
                    shl_instruction, shr_instruction,
                    jump_instruction, call_instruction,
                    jump_v0_instruction, rand_instruction,
                    add_no_overflow_instruction,
                    };
use crate::keypad::Keypad;
use crate::screen::{Screen, PixelState};
use crate::timers::{DelayTimer, SoundTimer, decrement_timer};

mod keypad;
mod registers;
mod opcodes;
mod screen;
mod timers;
 
type Stack<T> = Vec<T>;

fn execute_instruction(registers: &mut Chip8Registers, screen: &mut Screen,
                       keypad: &mut Keypad, memory: &mut [u8; 4096], stack: &mut Stack<u16>,
                       instruction: u16, program_counter: &mut usize,
                       delay_timer_arc: &Arc<Mutex<Box<dyn Timer + Send>>>,
                       sound_timer_arc: &Arc<Mutex<Box<dyn Timer + Send>>>)
{
    // Split the opcode into nibbles (4-bit values)
    let nibbles = (
        ((instruction & 0xF000) >> 12) as u8,
        ((instruction & 0x0F00) >> 8) as u8,
        ((instruction & 0x00F0) >> 4) as u8,
        (instruction & 0x000F) as u8,
    );

    match nibbles.0 {
        0x0 => match nibbles.3 {
            0x0 => {
                screen.clear_screen();
            },
            0xE => {
                let return_address = match stack.pop() {
                    Some(address) => address,
                    None => panic!("NO RETURN ADDRESS FOUND"),
                };
                *program_counter = return_address as usize;
            },
            _ => {
                println!("{:?}, {:?}", nibbles, instruction);
                panic!("NON VALID INSTRUCTION");
            },
        },
        0x1 => {
            let n1 = char::from_digit(nibbles.1 as u32, 16).unwrap().to_string();
            let n2 = char::from_digit(nibbles.2 as u32, 16).unwrap().to_string();
            let n3 = char::from_digit(nibbles.3 as u32, 16).unwrap().to_string();
            let nnn = u16::from_str_radix(&(n1 + &n2 + &n3), 16).unwrap();
            jump_instruction(program_counter, nnn);
        },
        0x2 => {
            let n1 = char::from_digit(nibbles.1 as u32, 16).unwrap().to_string();
            let n2 = char::from_digit(nibbles.2 as u32, 16).unwrap().to_string();
            let n3 = char::from_digit(nibbles.3 as u32, 16).unwrap().to_string();
            let nnn = u16::from_str_radix(&(n1 + &n2 + &n3), 16).unwrap();
            match call_instruction(program_counter, nnn, stack){
                Err(_) => panic!("ERROR: instruction failed at call instruction"),
                _ => {},
            };
        },
        0x3 => {
            let n1 = char::from_digit(nibbles.2 as u32, 16).unwrap().to_string();
            let n2 = char::from_digit(nibbles.3 as u32, 16).unwrap().to_string();
            let nn = u8::from_str_radix(&(n1 + &n2), 16).unwrap();
            let equals: bool = match nibbles.1 {
                0x0 => registers.v0.read() == nn,
                0x1 => registers.v1.read() == nn,
                0x2 => registers.v2.read() == nn,
                0x3 => registers.v3.read() == nn,
                0x4 => registers.v4.read() == nn,
                0x5 => registers.v5.read() == nn,
                0x6 => registers.v6.read() == nn,
                0x7 => registers.v7.read() == nn,
                0x8 => registers.v8.read() == nn,
                0x9 => registers.v9.read() == nn,
                0xA => registers.va.read() == nn,
                0xB => registers.vb.read() == nn,
                0xC => registers.vc.read() == nn,
                0xD => registers.vd.read() == nn,
                0xE => registers.ve.read() == nn,
                _ => {
                    println!("{:?}, {:?}", nibbles, instruction);
                    panic!("NON VALID INSTRUCTION");
                },
            };
            if equals {
                *program_counter += 2;
            }
        },
        0x4 => {
            let n1 = char::from_digit(nibbles.2 as u32, 16).unwrap().to_string();
            let n2 = char::from_digit(nibbles.3 as u32, 16).unwrap().to_string();
            let nn = u8::from_str_radix(&(n1 + &n2), 16).unwrap();
            let equals: bool = match nibbles.1 {
                0x0 => registers.v0.read() == nn,
                0x1 => registers.v1.read() == nn,
                0x2 => registers.v2.read() == nn,
                0x3 => registers.v3.read() == nn,
                0x4 => registers.v4.read() == nn,
                0x5 => registers.v5.read() == nn,
                0x6 => registers.v6.read() == nn,
                0x7 => registers.v7.read() == nn,
                0x8 => registers.v8.read() == nn,
                0x9 => registers.v9.read() == nn,
                0xA => registers.va.read() == nn,
                0xB => registers.vb.read() == nn,
                0xC => registers.vc.read() == nn,
                0xD => registers.vd.read() == nn,
                0xE => registers.ve.read() == nn,
                _ => {
                    println!("{:?}, {:?}", nibbles, instruction);
                    panic!("NON VALID INSTRUCTION");
                },
            };
            if !equals {
                *program_counter += 2;
            }
        },
        0x5 => {
            let vx_value = match nibbles.1 {
                0x0 => registers.v0.read(),
                0x1 => registers.v1.read(),
                0x2 => registers.v2.read(),
                0x3 => registers.v3.read(),
                0x4 => registers.v4.read(),
                0x5 => registers.v5.read(),
                0x6 => registers.v6.read(),
                0x7 => registers.v7.read(),
                0x8 => registers.v8.read(),
                0x9 => registers.v9.read(),
                0xA => registers.va.read(),
                0xB => registers.vb.read(),
                0xC => registers.vc.read(),
                0xD => registers.vd.read(),
                0xE => registers.ve.read(),
                _ => {
                    println!("{:?}, {:?}", nibbles, instruction);
                    panic!("NON VALID INSTRUCTION");
                },
            };
            let equals: bool = match nibbles.2 {
                0x0 => registers.v0.read() == vx_value,
                0x1 => registers.v1.read() == vx_value,
                0x2 => registers.v2.read() == vx_value,
                0x3 => registers.v3.read() == vx_value,
                0x4 => registers.v4.read() == vx_value,
                0x5 => registers.v5.read() == vx_value,
                0x6 => registers.v6.read() == vx_value,
                0x7 => registers.v7.read() == vx_value,
                0x8 => registers.v8.read() == vx_value,
                0x9 => registers.v9.read() == vx_value,
                0xA => registers.va.read() == vx_value,
                0xB => registers.vb.read() == vx_value,
                0xC => registers.vc.read() == vx_value,
                0xD => registers.vd.read() == vx_value,
                0xE => registers.ve.read() == vx_value,
                _ => {
                    println!("{:?}, {:?}", nibbles, instruction);
                    panic!("NON VALID INSTRUCTION");
                },
            };
            if equals {
                *program_counter += 2;
            }
        },
        0x6 => {
            let n1 = char::from_digit(nibbles.2 as u32, 16).unwrap().to_string();
            let n2 = char::from_digit(nibbles.3 as u32, 16).unwrap().to_string();
            let nn = u8::from_str_radix(&(n1 + &n2), 16).unwrap();
            match nibbles.1 {
                0x0 => registers.v0.write(nn),
                0x1 => registers.v1.write(nn),
                0x2 => registers.v2.write(nn),
                0x3 => registers.v3.write(nn),
                0x4 => registers.v4.write(nn),
                0x5 => registers.v5.write(nn),
                0x6 => registers.v6.write(nn),
                0x7 => registers.v7.write(nn),
                0x8 => registers.v8.write(nn),
                0x9 => registers.v9.write(nn),
                0xA => registers.va.write(nn),
                0xB => registers.vb.write(nn),
                0xC => registers.vc.write(nn),
                0xD => registers.vd.write(nn),
                0xE => registers.ve.write(nn),
                _ => {
                    println!("{:?}, {:?}", nibbles, instruction);
                    panic!("NON VALID INSTRUCTION");
                },
            };
        },
        0x7 => {
            let n1 = char::from_digit(nibbles.2 as u32, 16).unwrap().to_string();
            let n2 = char::from_digit(nibbles.3 as u32, 16).unwrap().to_string();
            let nn = u8::from_str_radix(&(n1 + &n2), 16).unwrap();
            match nibbles.1 {
                0x0 => add_no_overflow_instruction(&mut registers.v0, nn),
                0x1 => add_no_overflow_instruction(&mut registers.v1, nn),
                0x2 => add_no_overflow_instruction(&mut registers.v2, nn),
                0x3 => add_no_overflow_instruction(&mut registers.v3, nn),
                0x4 => add_no_overflow_instruction(&mut registers.v4, nn),
                0x5 => add_no_overflow_instruction(&mut registers.v5, nn),
                0x6 => add_no_overflow_instruction(&mut registers.v6, nn),
                0x7 => add_no_overflow_instruction(&mut registers.v7, nn),
                0x8 => add_no_overflow_instruction(&mut registers.v8, nn),
                0x9 => add_no_overflow_instruction(&mut registers.v9, nn),
                0xA => add_no_overflow_instruction(&mut registers.va, nn),
                0xB => add_no_overflow_instruction(&mut registers.vb, nn),
                0xC => add_no_overflow_instruction(&mut registers.vc, nn),
                0xD => add_no_overflow_instruction(&mut registers.vd, nn),
                0xE => add_no_overflow_instruction(&mut registers.ve, nn),
                _ => {
                    println!("{:?}, {:?}", nibbles, instruction);
                    panic!("NON VALID INSTRUCTION");
                }, 
            };
        },
        0x8 => match nibbles.3 {
            0x0 => {
                let vy_value = match nibbles.2 {
                    0x0 => registers.v0.read(),
                    0x1 => registers.v1.read(),
                    0x2 => registers.v2.read(),
                    0x3 => registers.v3.read(),
                    0x4 => registers.v4.read(),
                    0x5 => registers.v5.read(),
                    0x6 => registers.v6.read(),
                    0x7 => registers.v7.read(),
                    0x8 => registers.v8.read(),
                    0x9 => registers.v9.read(),
                    0xA => registers.va.read(),
                    0xB => registers.vb.read(),
                    0xC => registers.vc.read(),
                    0xD => registers.vd.read(),
                    0xE => registers.ve.read(),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                };
                match nibbles.1 {
                    0x0 => registers.v0.write(vy_value),
                    0x1 => registers.v1.write(vy_value),
                    0x2 => registers.v2.write(vy_value),
                    0x3 => registers.v3.write(vy_value),
                    0x4 => registers.v4.write(vy_value),
                    0x5 => registers.v5.write(vy_value),
                    0x6 => registers.v6.write(vy_value),
                    0x7 => registers.v7.write(vy_value),
                    0x8 => registers.v8.write(vy_value),
                    0x9 => registers.v9.write(vy_value),
                    0xA => registers.va.write(vy_value),
                    0xB => registers.vb.write(vy_value),
                    0xC => registers.vc.write(vy_value),
                    0xD => registers.vd.write(vy_value),
                    0xE => registers.ve.write(vy_value),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                }
            },
            0x1 => {
                let vy_value = match nibbles.2 {
                    0x0 => registers.v0.read(),
                    0x1 => registers.v1.read(),
                    0x2 => registers.v2.read(),
                    0x3 => registers.v3.read(),
                    0x4 => registers.v4.read(),
                    0x5 => registers.v5.read(),
                    0x6 => registers.v6.read(),
                    0x7 => registers.v7.read(),
                    0x8 => registers.v8.read(),
                    0x9 => registers.v9.read(),
                    0xA => registers.va.read(),
                    0xB => registers.vb.read(),
                    0xC => registers.vc.read(),
                    0xD => registers.vd.read(),
                    0xE => registers.ve.read(),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                };
                match nibbles.1 {
                    0x0 => registers.v0.write(registers.v0.read() | vy_value),
                    0x1 => registers.v1.write(registers.v1.read() | vy_value),
                    0x2 => registers.v2.write(registers.v2.read() | vy_value),
                    0x3 => registers.v3.write(registers.v3.read() | vy_value),
                    0x4 => registers.v4.write(registers.v4.read() | vy_value),
                    0x5 => registers.v5.write(registers.v5.read() | vy_value),
                    0x6 => registers.v6.write(registers.v6.read() | vy_value),
                    0x7 => registers.v7.write(registers.v7.read() | vy_value),
                    0x8 => registers.v8.write(registers.v8.read() | vy_value),
                    0x9 => registers.v9.write(registers.v9.read() | vy_value),
                    0xA => registers.va.write(registers.va.read() | vy_value),
                    0xB => registers.vb.write(registers.vb.read() | vy_value),
                    0xC => registers.vc.write(registers.vc.read() | vy_value),
                    0xD => registers.vd.write(registers.vd.read() | vy_value),
                    0xE => registers.ve.write(registers.ve.read() | vy_value),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                }
            },
            0x2 => {
                let vy_value = match nibbles.2 {
                    0x0 => registers.v0.read(),
                    0x1 => registers.v1.read(),
                    0x2 => registers.v2.read(),
                    0x3 => registers.v3.read(),
                    0x4 => registers.v4.read(),
                    0x5 => registers.v5.read(),
                    0x6 => registers.v6.read(),
                    0x7 => registers.v7.read(),
                    0x8 => registers.v8.read(),
                    0x9 => registers.v9.read(),
                    0xA => registers.va.read(),
                    0xB => registers.vb.read(),
                    0xC => registers.vc.read(),
                    0xD => registers.vd.read(),
                    0xE => registers.ve.read(),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                };
                match nibbles.1 {
                    0x0 => registers.v0.write(registers.v0.read() & vy_value),
                    0x1 => registers.v1.write(registers.v1.read() & vy_value),
                    0x2 => registers.v2.write(registers.v2.read() & vy_value),
                    0x3 => registers.v3.write(registers.v3.read() & vy_value),
                    0x4 => registers.v4.write(registers.v4.read() & vy_value),
                    0x5 => registers.v5.write(registers.v5.read() & vy_value),
                    0x6 => registers.v6.write(registers.v6.read() & vy_value),
                    0x7 => registers.v7.write(registers.v7.read() & vy_value),
                    0x8 => registers.v8.write(registers.v8.read() & vy_value),
                    0x9 => registers.v9.write(registers.v9.read() & vy_value),
                    0xA => registers.va.write(registers.va.read() & vy_value),
                    0xB => registers.vb.write(registers.vb.read() & vy_value),
                    0xC => registers.vc.write(registers.vc.read() & vy_value),
                    0xD => registers.vd.write(registers.vd.read() & vy_value),
                    0xE => registers.ve.write(registers.ve.read() & vy_value),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                }
            },
            0x3 => {
                let vy_value = match nibbles.2 {
                    0x0 => registers.v0.read(),
                    0x1 => registers.v1.read(),
                    0x2 => registers.v2.read(),
                    0x3 => registers.v3.read(),
                    0x4 => registers.v4.read(),
                    0x5 => registers.v5.read(),
                    0x6 => registers.v6.read(),
                    0x7 => registers.v7.read(),
                    0x8 => registers.v8.read(),
                    0x9 => registers.v9.read(),
                    0xA => registers.va.read(),
                    0xB => registers.vb.read(),
                    0xC => registers.vc.read(),
                    0xD => registers.vd.read(),
                    0xE => registers.ve.read(),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                };
                match nibbles.1 {
                    0x0 => registers.v0.write(registers.v0.read() ^ vy_value),
                    0x1 => registers.v1.write(registers.v1.read() ^ vy_value),
                    0x2 => registers.v2.write(registers.v2.read() ^ vy_value),
                    0x3 => registers.v3.write(registers.v3.read() ^ vy_value),
                    0x4 => registers.v4.write(registers.v4.read() ^ vy_value),
                    0x5 => registers.v5.write(registers.v5.read() ^ vy_value),
                    0x6 => registers.v6.write(registers.v6.read() ^ vy_value),
                    0x7 => registers.v7.write(registers.v7.read() ^ vy_value),
                    0x8 => registers.v8.write(registers.v8.read() ^ vy_value),
                    0x9 => registers.v9.write(registers.v9.read() ^ vy_value),
                    0xA => registers.va.write(registers.va.read() ^ vy_value),
                    0xB => registers.vb.write(registers.vb.read() ^ vy_value),
                    0xC => registers.vc.write(registers.vc.read() ^ vy_value),
                    0xD => registers.vd.write(registers.vd.read() ^ vy_value),
                    0xE => registers.ve.write(registers.ve.read() ^ vy_value),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                }
            },
            0x4 => {
                let vy_value = match nibbles.2 {
                    0x0 => registers.v0.read(),
                    0x1 => registers.v1.read(),
                    0x2 => registers.v2.read(),
                    0x3 => registers.v3.read(),
                    0x4 => registers.v4.read(),
                    0x5 => registers.v5.read(),
                    0x6 => registers.v6.read(),
                    0x7 => registers.v7.read(),
                    0x8 => registers.v8.read(),
                    0x9 => registers.v9.read(),
                    0xA => registers.va.read(),
                    0xB => registers.vb.read(),
                    0xC => registers.vc.read(),
                    0xD => registers.vd.read(),
                    0xE => registers.ve.read(),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                };
                match nibbles.1 {
                    0x0 => add_instruction(&mut registers.v0, vy_value, &mut registers.vf),
                    0x1 => add_instruction(&mut registers.v1, vy_value, &mut registers.vf),
                    0x2 => add_instruction(&mut registers.v2, vy_value, &mut registers.vf),
                    0x3 => add_instruction(&mut registers.v3, vy_value, &mut registers.vf),
                    0x4 => add_instruction(&mut registers.v4, vy_value, &mut registers.vf),
                    0x5 => add_instruction(&mut registers.v5, vy_value, &mut registers.vf),
                    0x6 => add_instruction(&mut registers.v6, vy_value, &mut registers.vf),
                    0x7 => add_instruction(&mut registers.v7, vy_value, &mut registers.vf),
                    0x8 => add_instruction(&mut registers.v8, vy_value, &mut registers.vf),
                    0x9 => add_instruction(&mut registers.v9, vy_value, &mut registers.vf),
                    0xA => add_instruction(&mut registers.va, vy_value, &mut registers.vf),
                    0xB => add_instruction(&mut registers.vb, vy_value, &mut registers.vf),
                    0xC => add_instruction(&mut registers.vc, vy_value, &mut registers.vf),
                    0xD => add_instruction(&mut registers.vd, vy_value, &mut registers.vf),
                    0xE => add_instruction(&mut registers.ve, vy_value, &mut registers.vf),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                }
            },
            0x5 => {
                let vy_value = match nibbles.2 {
                    0x0 => registers.v0.read(),
                    0x1 => registers.v1.read(),
                    0x2 => registers.v2.read(),
                    0x3 => registers.v3.read(),
                    0x4 => registers.v4.read(),
                    0x5 => registers.v5.read(),
                    0x6 => registers.v6.read(),
                    0x7 => registers.v7.read(),
                    0x8 => registers.v8.read(),
                    0x9 => registers.v9.read(),
                    0xA => registers.va.read(),
                    0xB => registers.vb.read(),
                    0xC => registers.vc.read(),
                    0xD => registers.vd.read(),
                    0xE => registers.ve.read(),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                };
                match nibbles.1 {
                    0x0 => sub_instruction(&mut registers.v0, vy_value, &mut registers.vf),
                    0x1 => sub_instruction(&mut registers.v1, vy_value, &mut registers.vf),
                    0x2 => sub_instruction(&mut registers.v2, vy_value, &mut registers.vf),
                    0x3 => sub_instruction(&mut registers.v3, vy_value, &mut registers.vf),
                    0x4 => sub_instruction(&mut registers.v4, vy_value, &mut registers.vf),
                    0x5 => sub_instruction(&mut registers.v5, vy_value, &mut registers.vf),
                    0x6 => sub_instruction(&mut registers.v6, vy_value, &mut registers.vf),
                    0x7 => sub_instruction(&mut registers.v7, vy_value, &mut registers.vf),
                    0x8 => sub_instruction(&mut registers.v8, vy_value, &mut registers.vf),
                    0x9 => sub_instruction(&mut registers.v9, vy_value, &mut registers.vf),
                    0xA => sub_instruction(&mut registers.va, vy_value, &mut registers.vf),
                    0xB => sub_instruction(&mut registers.vb, vy_value, &mut registers.vf),
                    0xC => sub_instruction(&mut registers.vc, vy_value, &mut registers.vf),
                    0xD => sub_instruction(&mut registers.vd, vy_value, &mut registers.vf),
                    0xE => sub_instruction(&mut registers.ve, vy_value, &mut registers.vf),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                }
            },
            0x6 => {
                match nibbles.1 {
                    0x0 => shr_instruction(&mut registers.v0, &mut registers.vf),
                    0x1 => shr_instruction(&mut registers.v1, &mut registers.vf),
                    0x2 => shr_instruction(&mut registers.v2, &mut registers.vf),
                    0x3 => shr_instruction(&mut registers.v3, &mut registers.vf),
                    0x4 => shr_instruction(&mut registers.v4, &mut registers.vf),
                    0x5 => shr_instruction(&mut registers.v5, &mut registers.vf),
                    0x6 => shr_instruction(&mut registers.v6, &mut registers.vf),
                    0x7 => shr_instruction(&mut registers.v7, &mut registers.vf),
                    0x8 => shr_instruction(&mut registers.v8, &mut registers.vf),
                    0x9 => shr_instruction(&mut registers.v9, &mut registers.vf),
                    0xA => shr_instruction(&mut registers.va, &mut registers.vf),
                    0xB => shr_instruction(&mut registers.vb, &mut registers.vf),
                    0xC => shr_instruction(&mut registers.vc, &mut registers.vf),
                    0xD => shr_instruction(&mut registers.vd, &mut registers.vf),
                    0xE => shr_instruction(&mut registers.ve, &mut registers.vf),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                }
            },
            0x7 => {
                let vy_value = match nibbles.2 {
                    0x0 => registers.v0.read(),
                    0x1 => registers.v1.read(),
                    0x2 => registers.v2.read(),
                    0x3 => registers.v3.read(),
                    0x4 => registers.v4.read(),
                    0x5 => registers.v5.read(),
                    0x6 => registers.v6.read(),
                    0x7 => registers.v7.read(),
                    0x8 => registers.v8.read(),
                    0x9 => registers.v9.read(),
                    0xA => registers.va.read(),
                    0xB => registers.vb.read(),
                    0xC => registers.vc.read(),
                    0xD => registers.vd.read(),
                    0xE => registers.ve.read(),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                };
                let vx_value = match nibbles.1 {
                    0x0 => registers.v0.read(),
                    0x1 => registers.v1.read(),
                    0x2 => registers.v2.read(),
                    0x3 => registers.v3.read(),
                    0x4 => registers.v4.read(),
                    0x5 => registers.v5.read(),
                    0x6 => registers.v6.read(),
                    0x7 => registers.v7.read(),
                    0x8 => registers.v8.read(),
                    0x9 => registers.v9.read(),
                    0xA => registers.va.read(),
                    0xB => registers.vb.read(),
                    0xC => registers.vc.read(),
                    0xD => registers.vd.read(),
                    0xE => registers.ve.read(),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                };
                let (result, is_borrow) = vy_value.overflowing_sub(vx_value);
                registers.vf.write((!is_borrow).into());
                match nibbles.1 {
                    0x0 => registers.v0.write(result),
                    0x1 => registers.v1.write(result),
                    0x2 => registers.v2.write(result),
                    0x3 => registers.v3.write(result),
                    0x4 => registers.v4.write(result),
                    0x5 => registers.v5.write(result),
                    0x6 => registers.v6.write(result),
                    0x7 => registers.v7.write(result),
                    0x8 => registers.v8.write(result),
                    0x9 => registers.v9.write(result),
                    0xA => registers.va.write(result),
                    0xB => registers.vb.write(result),
                    0xC => registers.vc.write(result),
                    0xD => registers.vd.write(result),
                    0xE => registers.ve.write(result),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                };
            },
            0xE => {
                match nibbles.1 {
                    0x0 => shl_instruction(&mut registers.v0, &mut registers.vf),
                    0x1 => shl_instruction(&mut registers.v1, &mut registers.vf),
                    0x2 => shl_instruction(&mut registers.v2, &mut registers.vf),
                    0x3 => shl_instruction(&mut registers.v3, &mut registers.vf),
                    0x4 => shl_instruction(&mut registers.v4, &mut registers.vf),
                    0x5 => shl_instruction(&mut registers.v5, &mut registers.vf),
                    0x6 => shl_instruction(&mut registers.v6, &mut registers.vf),
                    0x7 => shl_instruction(&mut registers.v7, &mut registers.vf),
                    0x8 => shl_instruction(&mut registers.v8, &mut registers.vf),
                    0x9 => shl_instruction(&mut registers.v9, &mut registers.vf),
                    0xA => shl_instruction(&mut registers.va, &mut registers.vf),
                    0xB => shl_instruction(&mut registers.vb, &mut registers.vf),
                    0xC => shl_instruction(&mut registers.vc, &mut registers.vf),
                    0xD => shl_instruction(&mut registers.vd, &mut registers.vf),
                    0xE => shl_instruction(&mut registers.ve, &mut registers.vf),
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                }
            },
            _ => {
                println!("{:?}, {:?}", nibbles, instruction);
                panic!("NON VALID INSTRUCTION");
            },
        },
        0x9 => {
            let vx_value = match nibbles.1 {
                0x0 => registers.v0.read(),
                0x1 => registers.v1.read(),
                0x2 => registers.v2.read(),
                0x3 => registers.v3.read(),
                0x4 => registers.v4.read(),
                0x5 => registers.v5.read(),
                0x6 => registers.v6.read(),
                0x7 => registers.v7.read(),
                0x8 => registers.v8.read(),
                0x9 => registers.v9.read(),
                0xA => registers.va.read(),
                0xB => registers.vb.read(),
                0xC => registers.vc.read(),
                0xD => registers.vd.read(),
                0xE => registers.ve.read(),
                _ => {
                    println!("{:?}, {:?}", nibbles, instruction);
                    panic!("NON VALID INSTRUCTION");
                },
            };
            let equals: bool = match nibbles.2 {
                0x0 => registers.v0.read() == vx_value,
                0x1 => registers.v1.read() == vx_value,
                0x2 => registers.v2.read() == vx_value,
                0x3 => registers.v3.read() == vx_value,
                0x4 => registers.v4.read() == vx_value,
                0x5 => registers.v5.read() == vx_value,
                0x6 => registers.v6.read() == vx_value,
                0x7 => registers.v7.read() == vx_value,
                0x8 => registers.v8.read() == vx_value,
                0x9 => registers.v9.read() == vx_value,
                0xA => registers.va.read() == vx_value,
                0xB => registers.vb.read() == vx_value,
                0xC => registers.vc.read() == vx_value,
                0xD => registers.vd.read() == vx_value,
                0xE => registers.ve.read() == vx_value,
                _ => {
                    println!("{:?}, {:?}", nibbles, instruction);
                    panic!("NON VALID INSTRUCTION");
                },
            };
            if !equals {
                *program_counter += 2;
            }
        },
        0xA => {
            let n1 = char::from_digit(nibbles.1 as u32, 16).unwrap().to_string();
            let n2 = char::from_digit(nibbles.2 as u32, 16).unwrap().to_string();
            let n3 = char::from_digit(nibbles.3 as u32, 16).unwrap().to_string();
            let nnn = u16::from_str_radix(&(n1 + &n2 + &n3), 16).unwrap();
            registers.i.write(nnn);
        },
        0xB => {
            let n1 = char::from_digit(nibbles.1 as u32, 16).unwrap().to_string();
            let n2 = char::from_digit(nibbles.2 as u32, 16).unwrap().to_string();
            let n3 = char::from_digit(nibbles.3 as u32, 16).unwrap().to_string();
            let nnn = u16::from_str_radix(&(n1 + &n2 + &n3), 16).unwrap();
            jump_v0_instruction(&registers.v0, program_counter, nnn);
        },
        0xC => {
            let n1 = char::from_digit(nibbles.2 as u32, 16).unwrap().to_string();
            let n2 = char::from_digit(nibbles.3 as u32, 16).unwrap().to_string();
            let nn = u8::from_str_radix(&(n1 + &n2), 16).unwrap();
            match nibbles.1 {
                0x0 => rand_instruction(&mut registers.v0, nn),
                0x1 => rand_instruction(&mut registers.v1, nn),
                0x2 => rand_instruction(&mut registers.v2, nn),
                0x3 => rand_instruction(&mut registers.v3, nn),
                0x4 => rand_instruction(&mut registers.v4, nn),
                0x5 => rand_instruction(&mut registers.v5, nn),
                0x6 => rand_instruction(&mut registers.v6, nn),
                0x7 => rand_instruction(&mut registers.v7, nn),
                0x8 => rand_instruction(&mut registers.v8, nn),
                0x9 => rand_instruction(&mut registers.v9, nn),
                0xA => rand_instruction(&mut registers.va, nn),
                0xB => rand_instruction(&mut registers.vb, nn),
                0xC => rand_instruction(&mut registers.vc, nn),
                0xD => rand_instruction(&mut registers.vd, nn),
                0xE => rand_instruction(&mut registers.ve, nn),
                _ => {
                    println!("{:?}, {:?}", nibbles, instruction);
                    panic!("NON VALID INSTRUCTION");
                },
            };
        },
        0xD => {
            let x_coordinate: u8 = match nibbles.1 {
                0x0 => registers.v0.read(),
                0x1 => registers.v1.read(),
                0x2 => registers.v2.read(),
                0x3 => registers.v3.read(),
                0x4 => registers.v4.read(),
                0x5 => registers.v5.read(),
                0x6 => registers.v6.read(),
                0x7 => registers.v7.read(),
                0x8 => registers.v8.read(),
                0x9 => registers.v9.read(),
                0xA => registers.va.read(),
                0xB => registers.vb.read(),
                0xC => registers.vc.read(),
                0xD => registers.vd.read(),
                0xE => registers.ve.read(),
                _ => {
                    println!("{:?}, {:?}", nibbles, instruction);
                    panic!("NON VALID INSTRUCTION");
                },
            };
            let y_coordinate: u8 = match nibbles.2 {
                0x0 => registers.v0.read(),
                0x1 => registers.v1.read(),
                0x2 => registers.v2.read(),
                0x3 => registers.v3.read(),
                0x4 => registers.v4.read(),
                0x5 => registers.v5.read(),
                0x6 => registers.v6.read(),
                0x7 => registers.v7.read(),
                0x8 => registers.v8.read(),
                0x9 => registers.v9.read(),
                0xA => registers.va.read(),
                0xB => registers.vb.read(),
                0xC => registers.vc.read(),
                0xD => registers.vd.read(),
                0xE => registers.ve.read(),
                _ => {
                    println!("{:?}, {:?}", nibbles, instruction);
                    panic!("NON VALID INSTRUCTION");
                },
            };
            registers.vf.write(0);
            let n: u8 = nibbles.3;

            let screen_width = 64;
            let screen_height = 32;

            let x_mod = x_coordinate % screen_width;
            let y_mod = y_coordinate % screen_height;

            for i in 0..n {
                let sprite_row  = memory[registers.i.read() as usize + i as usize] as u8;

                for j in (0..8).rev() {
                    let current_bit = (sprite_row >> j) & 1;
                    
                    let x_coord = (x_mod + (7 - j)) % screen_width;
                    let y_coord = (y_mod + i) % screen_height;

                    if current_bit == 1 {

                        let current_pixel = screen.get_pixel(&x_coord, &y_coord).expect("COORDINATES OUT OF BOUND");
                        match current_pixel {
                            PixelState::Off => screen.set_pixel(&x_coord, &y_coord, PixelState::On),
                            PixelState::On => {
                                screen.set_pixel(&x_coord, &y_coord, PixelState::Off);
                                registers.vf.write(1);
                            },
                        }
                    }
                }
            }
            execute!(std::io::stdout(), Clear(ClearType::All)).expect("ERROR CLEARING THE SCREEN");
            screen.display_pixels();
        },
        0xE => match nibbles.2 {
            0x9 => {
                let index = nibbles.1 as usize;
                if keypad.is_pressed(index) {
                    *program_counter += 2;
                }
            },
            0xA => {
                let index = nibbles.1 as usize;
                if !keypad.is_pressed(index) {
                    *program_counter += 2;
                }
            },
            _ => {
                println!("{:?}, {:?}", nibbles, instruction);
                panic!("NON VALID INSTRUCTION");
            },
        },
        0xF => match nibbles.2 {
            0x0 => match nibbles.3 {
                0x0 => {
                    match nibbles.1 {
                        0x0 => registers.v0.write(0),
                        0x1 => registers.v1.write(0),
                        0x2 => registers.v2.write(0),
                        0x3 => registers.v3.write(0),
                        0x4 => registers.v4.write(0),
                        0x5 => registers.v5.write(0),
                        0x6 => registers.v6.write(0),
                        0x7 => registers.v7.write(0),
                        0x8 => registers.v8.write(0),
                        0x9 => registers.v9.write(0),
                        0xA => registers.va.write(0),
                        0xB => registers.vb.write(0),
                        0xC => registers.vc.write(0),
                        0xD => registers.vd.write(0),
                        0xE => registers.ve.write(0),
                        0xF => registers.vf.write(0),
                        _ => {
                            println!("{:?}, {:?}", nibbles, instruction);
                            panic!("NON VALID INSTRUCTION");
                        },
                    }
                },
                0x7 => {
                    let delay_timer = delay_timer_arc.lock().expect("Failed to lock delay timer");
                    match nibbles.1 {
                        0x0 => registers.v0.write(delay_timer.get_timer()),
                        0x1 => registers.v1.write(delay_timer.get_timer()),
                        0x2 => registers.v2.write(delay_timer.get_timer()),
                        0x3 => registers.v3.write(delay_timer.get_timer()),
                        0x4 => registers.v4.write(delay_timer.get_timer()),
                        0x5 => registers.v5.write(delay_timer.get_timer()),
                        0x6 => registers.v6.write(delay_timer.get_timer()),
                        0x7 => registers.v7.write(delay_timer.get_timer()),
                        0x8 => registers.v8.write(delay_timer.get_timer()),
                        0x9 => registers.v9.write(delay_timer.get_timer()),
                        0xA => registers.va.write(delay_timer.get_timer()),
                        0xB => registers.vb.write(delay_timer.get_timer()),
                        0xC => registers.vc.write(delay_timer.get_timer()),
                        0xD => registers.vd.write(delay_timer.get_timer()),
                        0xE => registers.ve.write(delay_timer.get_timer()),
                        _ => {
                            println!("{:?}, {:?}", nibbles, instruction);
                            panic!("NON VALID INSTRUCTION");
                        },
                    }
                    drop(delay_timer);
                },
                0xA => {
                    loop {
                        let mut input = [0u8; 1];
                        match std::io::stdin().read_exact(&mut input) {
                            Ok(_) => {
                                let key: u8 = match input[0] {
                                    b'1' => 0x1,
                                    b'2' => 0x2,
                                    b'3' => 0x3,
                                    b'4' => 0xC,
                                    b'q' => 0x4,
                                    b'w' => 0x5,
                                    b'e' => 0x6,
                                    b'r' => 0xD,
                                    b'a' => 0x7,
                                    b's' => 0x8,
                                    b'd' => 0x9,
                                    b'f' => 0xE,
                                    b'z' => 0xA,
                                    b'x' => 0x0,
                                    b'c' => 0xB,
                                    b'v' => 0xF,
                                    _ => continue, // ignore other keys
                                };
                                match nibbles.1 {
                                    0x0 => registers.v0.write(key),
                                    0x1 => registers.v1.write(key),
                                    0x2 => registers.v2.write(key),
                                    0x3 => registers.v3.write(key),
                                    0x4 => registers.v4.write(key),
                                    0x5 => registers.v5.write(key),
                                    0x6 => registers.v6.write(key),
                                    0x7 => registers.v7.write(key),
                                    0x8 => registers.v8.write(key),
                                    0x9 => registers.v9.write(key),
                                    0xA => registers.va.write(key),
                                    0xB => registers.vb.write(key),
                                    0xC => registers.vc.write(key),
                                    0xD => registers.vd.write(key),
                                    0xE => registers.ve.write(key),
                                    _ => {
                                        println!("{:?}, {:?}", nibbles, instruction);
                                        panic!("NON VALID INSTRUCTION");
                                    },
                                }
                                keypad.press_key(key as usize);
                                for i in 0..16 {
                                    if i == key {
                                        continue;
                                    }
                                    keypad.release_key(i as usize);
                                }
                                break;
                            }
                            Err(_) => continue, // ignore errors
                        }
                    }
                },
                _ => {
                    println!("{:?}, {:?}", nibbles, instruction);
                    panic!("NON VALID INSTRUCTION");
                },
            },
            0x1 => match nibbles.3 {
                0x5 => {
                    let mut delay_timer = delay_timer_arc.lock().expect("Failed to lock delay timer");
                    match nibbles.1 {
                        0x0 => delay_timer.set_timer(registers.v0.read()),
                        0x1 => delay_timer.set_timer(registers.v1.read()),
                        0x2 => delay_timer.set_timer(registers.v2.read()),
                        0x3 => delay_timer.set_timer(registers.v3.read()),
                        0x4 => delay_timer.set_timer(registers.v4.read()),
                        0x5 => delay_timer.set_timer(registers.v5.read()),
                        0x6 => delay_timer.set_timer(registers.v6.read()),
                        0x7 => delay_timer.set_timer(registers.v7.read()),
                        0x8 => delay_timer.set_timer(registers.v8.read()),
                        0x9 => delay_timer.set_timer(registers.v9.read()),
                        0xA => delay_timer.set_timer(registers.va.read()),
                        0xB => delay_timer.set_timer(registers.vb.read()),
                        0xC => delay_timer.set_timer(registers.vc.read()),
                        0xD => delay_timer.set_timer(registers.vd.read()),
                        0xE => delay_timer.set_timer(registers.ve.read()),
                        _ => {
                            println!("{:?}, {:?}", nibbles, instruction);
                            panic!("NON VALID INSTRUCTION");
                        },
                    };
                    drop(delay_timer);
                },
                0x8 => {
                    let mut sound_timer = sound_timer_arc.lock().expect("Failed to lock delay timer");
                    match nibbles.1 {
                        0x0 => sound_timer.set_timer(registers.v0.read()),
                        0x1 => sound_timer.set_timer(registers.v1.read()),
                        0x2 => sound_timer.set_timer(registers.v2.read()),
                        0x3 => sound_timer.set_timer(registers.v3.read()),
                        0x4 => sound_timer.set_timer(registers.v4.read()),
                        0x5 => sound_timer.set_timer(registers.v5.read()),
                        0x6 => sound_timer.set_timer(registers.v6.read()),
                        0x7 => sound_timer.set_timer(registers.v7.read()),
                        0x8 => sound_timer.set_timer(registers.v8.read()),
                        0x9 => sound_timer.set_timer(registers.v9.read()),
                        0xA => sound_timer.set_timer(registers.va.read()),
                        0xB => sound_timer.set_timer(registers.vb.read()),
                        0xC => sound_timer.set_timer(registers.vc.read()),
                        0xD => sound_timer.set_timer(registers.vd.read()),
                        0xE => sound_timer.set_timer(registers.ve.read()),
                        _ => {
                            println!("{:?}, {:?}", nibbles, instruction);
                            panic!("NON VALID INSTRUCTION");
                        },
                    };
                    drop(sound_timer);
                },
                0xE => {
                    let (result, _overflow) = match nibbles.1 {
                        0x0 => registers.i.read().overflowing_add(registers.v0.read() as u16),
                        0x1 => registers.i.read().overflowing_add(registers.v1.read() as u16),
                        0x2 => registers.i.read().overflowing_add(registers.v2.read() as u16),
                        0x3 => registers.i.read().overflowing_add(registers.v3.read() as u16),
                        0x4 => registers.i.read().overflowing_add(registers.v4.read() as u16),
                        0x5 => registers.i.read().overflowing_add(registers.v5.read() as u16),
                        0x6 => registers.i.read().overflowing_add(registers.v6.read() as u16),
                        0x7 => registers.i.read().overflowing_add(registers.v7.read() as u16),
                        0x8 => registers.i.read().overflowing_add(registers.v8.read() as u16),
                        0x9 => registers.i.read().overflowing_add(registers.v9.read() as u16),
                        0xA => registers.i.read().overflowing_add(registers.va.read() as u16),
                        0xB => registers.i.read().overflowing_add(registers.vb.read() as u16),
                        0xC => registers.i.read().overflowing_add(registers.vc.read() as u16),
                        0xD => registers.i.read().overflowing_add(registers.vd.read() as u16),
                        0xE => registers.i.read().overflowing_add(registers.ve.read() as u16),
                        _ => {
                            println!("{:?}, {:?}", nibbles, instruction);
                            panic!("NON VALID INSTRUCTION");
                        },
                    };
                    registers.i.write(result);
                },
                _ => {
                    println!("{:?}, {:?}", nibbles, instruction);
                    panic!("NON VALID INSTRUCTION");
                },
            },
            0x2 => {
                let address: u16 = match nibbles.1 {
                    0x0 => registers.v0.read() as u16,
                    0x1 => registers.v1.read() as u16,
                    0x2 => registers.v2.read() as u16,
                    0x3 => registers.v3.read() as u16,
                    0x4 => registers.v4.read() as u16,
                    0x5 => registers.v5.read() as u16,
                    0x6 => registers.v6.read() as u16,
                    0x7 => registers.v7.read() as u16,
                    0x8 => registers.v8.read() as u16,
                    0x9 => registers.v9.read() as u16,
                    0xA => registers.va.read() as u16,
                    0xB => registers.vb.read() as u16,
                    0xC => registers.vc.read() as u16,
                    0xD => registers.vd.read() as u16,
                    0xE => registers.ve.read() as u16,
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                };
                registers.i.write(address * 5);
            },
            0x3 => {
                let value: i32 = match nibbles.1 {
                    0x0 => registers.v0.read() as i32,
                    0x1 => registers.v1.read() as i32,
                    0x2 => registers.v2.read() as i32,
                    0x3 => registers.v3.read() as i32,
                    0x4 => registers.v4.read() as i32,
                    0x5 => registers.v5.read() as i32,
                    0x6 => registers.v6.read() as i32,
                    0x7 => registers.v7.read() as i32,
                    0x8 => registers.v8.read() as i32,
                    0x9 => registers.v9.read() as i32,
                    0xA => registers.va.read() as i32,
                    0xB => registers.vb.read() as i32,
                    0xC => registers.vc.read() as i32,
                    0xD => registers.vd.read() as i32,
                    0xE => registers.ve.read() as i32,
                    _ => {
                        println!("{:?}, {:?}", nibbles, instruction);
                        panic!("NON VALID INSTRUCTION");
                    },
                };
                memory[registers.i.read() as usize] = (value / 100) as u8;
                memory[(registers.i.read() + 1) as usize] = (value / 10 % 10) as u8;
                memory[(registers.i.read() + 2) as usize] = (value % 10) as u8; 
            },
            0x5=> {
                for i in 0..=nibbles.1 {
                    let memory_index = (registers.i.read() + i as u16) as usize;
                    memory[memory_index] = match i {
                        0x0 => registers.v0.read(),
                        0x1 => registers.v1.read(),
                        0x2 => registers.v2.read(),
                        0x3 => registers.v3.read(),
                        0x4 => registers.v4.read(),
                        0x5 => registers.v5.read(),
                        0x6 => registers.v6.read(),
                        0x7 => registers.v7.read(),
                        0x8 => registers.v8.read(),
                        0x9 => registers.v9.read(),
                        0xA => registers.va.read(),
                        0xB => registers.vb.read(),
                        0xC => registers.vc.read(),
                        0xD => registers.vd.read(),
                        0xE => registers.ve.read(),
                        _ => {
                            println!("{:?}, {:?}", nibbles, instruction);
                            panic!("NON VALID INSTRUCTION");
                        },
                    };
                }
            },
            0x6 => {
                let mut index = registers.i.read() as usize;
                for i in 0..=nibbles.1 {
                    let current_value = memory[index];
                    match i {
                        0x0 => registers.v0.write(current_value),
                        0x1 => registers.v1.write(current_value),
                        0x2 => registers.v2.write(current_value),
                        0x3 => registers.v3.write(current_value),
                        0x4 => registers.v4.write(current_value),
                        0x5 => registers.v5.write(current_value),
                        0x6 => registers.v6.write(current_value),
                        0x7 => registers.v7.write(current_value),
                        0x8 => registers.v8.write(current_value),
                        0x9 => registers.v9.write(current_value),
                        0xA => registers.va.write(current_value),
                        0xB => registers.vb.write(current_value),
                        0xC => registers.vc.write(current_value),
                        0xD => registers.vd.write(current_value),
                        0xE => registers.ve.write(current_value),
                        _ => {
                            println!("{:?}, {:?}", nibbles, instruction);
                            panic!("NON VALID INSTRUCTION");
                        },
                    };
                    index += 1;
                }
            },
            _ => {
                println!("{:?}, {:?}", nibbles, instruction);
                panic!("NON VALID INSTRUCTION");
            }, 
        },
        _ => {
            println!("{:?}, {:?}", nibbles, instruction);
            panic!("NON VALID INSTRUCTION");
        },
    }
}

fn load_file_to_memory(memory: &mut [u8], file_path: &str, start_address: usize) -> Result<()> {
    let mut file = File::open(file_path)?;
    let mut buffer = [0; 2];
    let mut address = start_address;

    loop {
        match file.read_exact(&mut buffer) {
            Ok(_) => {
                let opcode = u16::from_be_bytes(buffer);
                memory[address] = ((opcode >> 8) & 0xFF) as u8;
                memory[address + 1] = (opcode & 0xFF) as u8;

                address += 2;
            }
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

fn main() {

    let mut registers: Chip8Registers = Chip8Registers::new(); // registers v0 - vf + i
    let mut stack: Stack<u16> = Stack::new(); // stack of addresses
    let mut screen: Screen = Screen::new(); // set the screen pixels to all off  
    let mut memory: [u8; 4096] = [0; 4096]; // set the memory
    let mut keypad: Keypad = Keypad::new();
    let mut program_counter: usize = 0x200;

    let delay_timer = DelayTimer::new();
    let sound_timer = SoundTimer::new();

    let delay_timer_arc = Arc::new(Mutex::new(Box::new(delay_timer) as Box<dyn Timer + Send>));
    let sound_timer_arc = Arc::new(Mutex::new(Box::new(sound_timer) as Box<dyn Timer + Send>));

    let delay_timer_arc_clone = delay_timer_arc.clone();
    let sound_timer_arc_clone = sound_timer_arc.clone();

    thread::spawn(move || decrement_timer(delay_timer_arc_clone));
    thread::spawn(move || decrement_timer(sound_timer_arc_clone));

    let font: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
                          0x20, 0x60, 0x20, 0x20, 0x70, // 1
                          0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
                          0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
                          0x90, 0x90, 0xF0, 0x10, 0x10, // 4
                          0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
                          0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
                          0xF0, 0x10, 0x20, 0x40, 0x40, // 7
                          0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
                          0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
                          0xF0, 0x90, 0xF0, 0x90, 0x90, // A
                          0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
                          0xF0, 0x80, 0x80, 0x80, 0xF0, // C
                          0xE0, 0x90, 0x90, 0x90, 0xE0, // D
                          0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
                          0xF0, 0x80, 0xF0, 0x80, 0x80]; // F

    // setting the font, and pointing I to it
    for (i ,font_byte) in font.iter().enumerate() {
        memory[i as usize + 0x50 as usize] = *font_byte;
    }
    registers.i.write(0x50);
    
    match load_file_to_memory(&mut memory, "C:\\Users\\urits\\Downloads\\Chip8Picture.ch8", 0x200){
        Ok(_) => {},
        Err(_) => panic!("FAILED TO LOAD ROM TO MEMORY"),
    }

    loop {
        let current_instruction: u16 = (u16::from(memory[program_counter]) << 8) | u16::from(memory[program_counter + 1]);
        if (program_counter >= memory.len()) || (current_instruction == 65535) {
            break;
        }
        execute_instruction(&mut registers, &mut screen, &mut keypad,
                            &mut memory, &mut stack, current_instruction,
                            &mut program_counter, &delay_timer_arc,
                            &sound_timer_arc);
        
        // get the next instruction from memory
        program_counter += 2;
        thread::sleep(Duration::from_millis(100));
    }
}