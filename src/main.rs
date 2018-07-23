#![feature(box_syntax)]

use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use std::env;
use std::process;
use std::collections::VecDeque;

const MEMORY_SIZE: usize = 65536;

enum Instruction {
    Next,
    Previous,
    Increment,
    Decrement,
    Output,
    Input,
    // The two bracket instructions carry the index of the matching bracket with them.
    Open(usize),
    Close(usize)
}

struct Processor {
    pointer: usize,
    pc: usize,
    program: Vec<Instruction>,
    memory: [u8; MEMORY_SIZE],
    input_buffer: VecDeque<u8>
}

impl Processor {
    fn step(&mut self) {
        let instruction = &self.program[self.pc];
        self.pc += 1;

        match instruction {
            Instruction::Next           => self.pointer += 1,
            Instruction::Previous       => self.pointer -= 1,
            Instruction::Increment      => self.memory[self.pointer] = self.memory[self.pointer].wrapping_add(1),
            Instruction::Decrement      => self.memory[self.pointer] = self.memory[self.pointer].wrapping_sub(1),
            Instruction::Output         => {
                print!("{}", self.memory[self.pointer] as char);
                io::stdout().flush().ok().expect("The output could not be flushed.");
            },
            Instruction::Input          => {
                let mut byte: Option<u8> = None;
                while byte.is_none() {
                    let next = self.input_buffer.pop_front();
                    if next.is_some() {
                        byte = next;
                    } else {
                        let mut line_buffer = String::new();
                        io::stdin().read_line(&mut line_buffer).ok().expect("The console could not be read.");
                        for line_byte in line_buffer.bytes() {
                            self.input_buffer.push_back(line_byte);
                        }
                    }
                }
                self.memory[self.pointer] = byte.expect("The input could not be written to memory.");
            },
            Instruction::Open(close)    => {
                if self.memory[self.pointer] == 0 {
                    self.pc = close + 1;
                }
            },
            Instruction::Close(open)    => {
                if self.memory[self.pointer] != 0 {
                    self.pc = open + 1;
                }
            }
        }
    }

    fn run(&mut self) {
        while self.pc < self.program.len() {
            self.step();
        }
    }

   fn new() -> Processor {
	Processor {
	    pointer:        0,
	    pc:             0,
	    program:        Vec::new(),
	    memory:         [0; MEMORY_SIZE],
	    input_buffer:   VecDeque::new()
	}
   }
}

fn decode(code: &str) -> Vec<Instruction> {
    let mut instructions: Vec<Instruction> = Vec::with_capacity(code.len());
    let mut open_stack: Vec<usize> = Vec::new();
    let mut i: usize = 0;

    for character in code.chars() {
        let instruction = match character {
                '>' => Instruction::Next,
                '<' => Instruction::Previous,
                '+' => Instruction::Increment,
                '-' => Instruction::Decrement,
                '.' => Instruction::Output,
                ',' => Instruction::Input,
                '[' => {
                    open_stack.push(i);
                    Instruction::Open(0) // This gets overwritten when the matching bracket is found.
                },
                ']' => {
                    let open_bracket = open_stack.pop().unwrap();
                    instructions[open_bracket] = Instruction::Open(i);
                    Instruction::Close(open_bracket)
                },
                _ => continue
            };
        instructions.push(instruction);
        i += 1;
    }
    instructions
}

fn main() {
    let arguments = env::args_os().map(PathBuf::from).collect::<Vec<PathBuf>>();
    if arguments.len() < 2 {
        println!("Usage: neuro-bf [program]");
        process::exit(1);
    }
    let filename = &arguments[1];

    let mut file = File::open(filename).expect("\"{}\" could not be found.");
    let mut program_str = String::new();
    file.read_to_string(&mut program_str).expect("\"{}\" could not be read.");

    let mut processor = box Processor::new();
    processor.program = decode(&program_str);
    processor.run();    
}
