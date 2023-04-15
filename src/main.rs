use core::panic;
use std::thread;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::{Read, Result, ErrorKind, self};
use std::time::Duration;
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
};
use timers::Timer;
use num::cast::FromPrimitive;
use std::path::PathBuf;

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
mod opcodes;
mod screen;
mod timers;
 
type Stack<T> = Vec<T>;

fn execute_instruction(registers: &mut [u8; 16], register_i: &mut u16,
                       screen: &mut Screen,
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
            let equals: bool = (*registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER")) == nn;
            if equals {
                *program_counter += 2;
            }
        },
        0x4 => {
            let n1 = char::from_digit(nibbles.2 as u32, 16).unwrap().to_string();
            let n2 = char::from_digit(nibbles.3 as u32, 16).unwrap().to_string();
            let nn = u8::from_str_radix(&(n1 + &n2), 16).unwrap();
            let equals: bool = *registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER") == nn;
            if !equals {
                *program_counter += 2;
            }
        },
        0x5 => {
            let vx_value: u8 = *registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER");
            let equals: bool = *registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER") == vx_value;
            if equals {
                *program_counter += 2;
            }
        },
        0x6 => {
            let n1 = char::from_digit(nibbles.2 as u32, 16).unwrap().to_string();
            let n2 = char::from_digit(nibbles.3 as u32, 16).unwrap().to_string();
            let nn = u8::from_str_radix(&(n1 + &n2), 16).unwrap();
            registers[nibbles.1 as usize] = nn;
        },
        0x7 => {
            let n1 = char::from_digit(nibbles.2 as u32, 16).unwrap().to_string();
            let n2 = char::from_digit(nibbles.3 as u32, 16).unwrap().to_string();
            let nn = u8::from_str_radix(&(n1 + &n2), 16).unwrap();
            add_no_overflow_instruction(registers, nibbles.1.into(), nn);
        },
        0x8 => match nibbles.3 {
            0x0 => {
                let vy_value: u8 = *registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER");
                registers[nibbles.1 as usize] = vy_value;
            },
            0x1 => {
                let vy_value: u8 = *registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER");
                registers[nibbles.1 as usize] = registers[nibbles.1 as usize] | vy_value;
            },
            0x2 => {
                let vy_value: u8 = *registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER");
                registers[nibbles.1 as usize] = registers[nibbles.1 as usize] & vy_value;
            },
            0x3 => {
                let vy_value: u8 = *registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER");
                registers[nibbles.1 as usize] = registers[nibbles.1 as usize] ^ vy_value;
            },
            0x4 => {
                let vy_value: u8 = *registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER");
                add_instruction(registers, nibbles.1.into(), vy_value);
            },
            0x5 => {
                let vy_value: u8 = *registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER");
                sub_instruction(registers, nibbles.1.into(), vy_value);
            },
            0x6 => {
                shr_instruction(registers, nibbles.1.into());
            },
            0x7 => {
                let vy_value: u8 = *registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER");
                let vx_value: u8 = *registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER");
                let (result, is_borrow) = vy_value.overflowing_sub(vx_value);
                registers[0xF as usize] = (!is_borrow).into();
                registers[nibbles.1 as usize] = result;
            },
            0xE => {
                shl_instruction(registers, nibbles.1.into());
            },
            _ => {
                println!("{:?}, {:?}", nibbles, instruction);
                panic!("NON VALID INSTRUCTION");
            },
        },
        0x9 => {
            let vx_value: u8 = *registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER");
            let equals = (*registers.get::<usize>(nibbles.2.into()).expect("NON VALID REGISTER")) == vx_value;
            if !equals {
                *program_counter += 2;
            }
        },
        0xA => {
            let n1 = char::from_digit(nibbles.1 as u32, 16).unwrap().to_string();
            let n2 = char::from_digit(nibbles.2 as u32, 16).unwrap().to_string();
            let n3 = char::from_digit(nibbles.3 as u32, 16).unwrap().to_string();
            let nnn = u16::from_str_radix(&(n1 + &n2 + &n3), 16).unwrap();
            *register_i = nnn;
        },
        0xB => {
            let n1 = char::from_digit(nibbles.1 as u32, 16).unwrap().to_string();
            let n2 = char::from_digit(nibbles.2 as u32, 16).unwrap().to_string();
            let n3 = char::from_digit(nibbles.3 as u32, 16).unwrap().to_string();
            let nnn = u16::from_str_radix(&(n1 + &n2 + &n3), 16).unwrap();
            jump_v0_instruction(*registers, program_counter, nnn);
        },
        0xC => {
            let n1 = char::from_digit(nibbles.2 as u32, 16).unwrap().to_string();
            let n2 = char::from_digit(nibbles.3 as u32, 16).unwrap().to_string();
            let nn = u8::from_str_radix(&(n1 + &n2), 16).unwrap();
            rand_instruction(registers, nibbles.1.into(), nn);
        },
        0xD => {
            let screen_width = 64;
            let screen_height = 32;

            let x_coordinate: u8 = *registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER");
            let y_coordinate: u8 = *registers.get::<usize>(nibbles.2.into()).expect("NON VALID REGISTER");
            registers[0xF as usize] = 0;
            let n: u8 = nibbles.3;

            let x_mod: u8 = x_coordinate % screen_width;
            let y_mod: u8 = y_coordinate % screen_height;

            for i in 0..n {
                let sprite_row  = memory[(*register_i + u16::from(i)) as usize] as u8;

                for j in (0..8).rev() {
                    let current_bit = (sprite_row >> j) & 1;
                    
                    let x_coord: u8 = (x_mod + (7 - j)) % screen_width;
                    let y_coord: u8 = (y_mod + i) % screen_height;

                    if current_bit == 1 {

                        let current_pixel = screen.get_pixel(&x_coord, &y_coord).expect("COORDINATES OUT OF BOUND");
                        match current_pixel {
                            PixelState::Off => screen.set_pixel(&x_coord, &y_coord, PixelState::On),
                            PixelState::On => {
                                screen.set_pixel(&x_coord, &y_coord, PixelState::Off);
                                registers[0xF as usize] = 1;
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
                let index = nibbles.1.into();
                if keypad.is_pressed(index) {
                    *program_counter += 2;
                }
            },
            0xA => {
                let index = nibbles.1.into();
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
                    registers[nibbles.1 as usize] = 0;
                },
                0x7 => {
                    let delay_timer = delay_timer_arc.lock().expect("Failed to lock delay timer");
                    registers[nibbles.1 as usize] = delay_timer.get_timer();
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
                                registers[nibbles.1 as usize] = key;
                                keypad.press_key(key.into());
                                for i in 0..16 {
                                    if i == key {
                                        continue;
                                    }
                                    keypad.release_key(i.into());
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
                    delay_timer.set_timer(*registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER"));
                    drop(delay_timer);
                },
                0x8 => {
                    let mut sound_timer = sound_timer_arc.lock().expect("Failed to lock delay timer");
                    sound_timer.set_timer(*registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER"));
                    drop(sound_timer);
                },
                0xE => {
                    let (result, _overflow) = register_i.overflowing_add((*registers.get::<usize>(nibbles.1.into())
                                                        .expect("NON VALID REGISTER")).into());
                    *register_i = result;
                },
                _ => {
                    println!("{:?}, {:?}", nibbles, instruction);
                    panic!("NON VALID INSTRUCTION");
                },
            },
            0x2 => {
                let address: u16 = FromPrimitive::from_u8(*registers.get::<usize>(nibbles.1 as usize).expect("NON VALID REGISTER")).unwrap();
                *register_i = address * 5; 
            },
            0x3 => {
                let value: i32 = i32::from(*registers.get::<usize>(nibbles.1.into()).expect("NON VALID REGISTER")); 
                memory[(*register_i) as usize] = (value / 100) as u8;
                memory[(*register_i + 1) as usize] = (value / 10 % 10) as u8;
                memory[(*register_i + 2) as usize] = (value % 10) as u8; 
            },
            0x5=> {
                for i in 0..=nibbles.1 {
                    let memory_index: usize = (*register_i + u16::from(i)) as usize;
                    memory[memory_index] = *registers.get::<usize>(i as usize).expect("NON VALID REGISTER");
                }
            },
            0x6 => {
                let mut index: usize = (*register_i) as usize;
                for i in 0..=nibbles.1 {
                    let current_value = memory[index];
                    registers[i as usize] = current_value;
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

fn get_path_from_user() -> PathBuf {
    println!("Enter the path to the ROM of your program:");
    let mut read_path = String::new();
    io::stdin().read_line(&mut read_path).expect("FAILED READING PATH FROM THE USER");
    let file_path = read_path.trim();

    // Use the `Path` module to create a `PathBuf` from the file path
    let path = std::path::Path::new(&file_path).to_owned();

    // Check if the file exists using the `fs` module
    if path.exists() && path.is_file() {
    } else {
        panic!("THE FILE IS NOT FOUND");
    }
    path
}

fn main() {
    let mut registers: [u8; 16] = [0u8; 16]; // registers v0 - vf 
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

    let mut register_i  = 0x50; // register 16 = i

    let path = get_path_from_user();
    
    match load_file_to_memory(&mut memory, path.to_str().expect("COULDN'T CONVERET PATH TO &str"), 0x200){
        Ok(_) => {},
        Err(_) => panic!("FAILED TO LOAD ROM TO MEMORY"),
    }

    loop {
        let current_instruction: u16 = (u16::from(memory[program_counter]) << 8) | u16::from(memory[program_counter + 1]);
        if (program_counter >= memory.len()) || (current_instruction == 65535) {
            break;
        }
        execute_instruction(&mut registers, &mut register_i, &mut screen, &mut keypad,
                            &mut memory, &mut stack, current_instruction,
                            &mut program_counter, &delay_timer_arc,
                            &sound_timer_arc);
        
        // get::<usize> the next instruction from memory
        program_counter += 2;
        thread::sleep(Duration::from_millis(100));
    }
}