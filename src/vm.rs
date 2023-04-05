use std::collections::VecDeque;

use crate::error::SynacorErr;
use crate::opcodes::{OpName, INS_WIDTH};
use std::{thread, time};

const BITS_15: usize = 32768;

#[derive(Debug)]
pub struct VM {
    memory: [u16; BITS_15],
    stack: Vec<u16>,
    registers: [u16; 8],
    addr: usize,
    input: VecDeque<u16>,
    auto: bool,
    auto_commands: Vec<String>,
}

impl VM {
    pub fn new(bytes: Vec<u16>, auto: bool) -> Self {
        let mut memory = [0; 32768];

        // leave trailing zeroes past values read into memory
        memory[..bytes.len()].copy_from_slice(&bytes[..]);

        Self {
            memory,
            stack: Vec::new(),
            registers: [0; 8],
            addr: 0,
            input: VecDeque::new(),
            auto,
            auto_commands: SOLUTION.iter().rev().map(|&s| s.into()).collect(),
        }
    }

    // generic so that it can handle the main loop and intermediate errors
    fn err<T>(&self, details: String) -> Result<T, SynacorErr> {
        Err(SynacorErr::new_addr(self.addr, details))
    }

    fn read_mem(&self, offset: usize) -> Result<u16, SynacorErr> {
        if let Some(val) = self.memory.get(self.addr + offset) {
            let val_addr = *val as usize;

            if val_addr < BITS_15 {
                Ok(*val)
            } else if val_addr < BITS_15 + 8 {
                Ok(self.registers[val_addr - BITS_15])
            } else {
                Err(SynacorErr::new_addr(
                    self.addr + offset,
                    format!("Value {} falls outside 15-bit range.", val),
                ))
            }
        } else {
            self.err(format!(
                "Attempt to access invalid memory address {}",
                self.addr + offset
            ))
        }
    }

    fn assign_reg(&mut self, new_val: u16) -> Result<(), SynacorErr> {
        match self.memory.get(self.addr + 1) {
            Some(reg) => {
                if (32768..32775).contains(reg) {
                    *&mut self.registers[(*reg as usize) - BITS_15] = new_val;
                    Ok(())
                } else {
                    self.err(format!("Attempted to access invalid register {}", *reg))
                }
            }
            None => self.err(format!(
                "Attempt to access invalid memory address {}",
                self.addr + 1
            )),
        }
    }

    // returns true if we should halt
    fn step(&mut self) -> Result<bool, SynacorErr> {
        let opcode_id = self.read_mem(0)?;

        if let (Some(width), Ok(opname)) = (INS_WIDTH.get(&opcode_id), OpName::try_from(opcode_id))
        {
            // not all opcodes will need or have valid a, b, c for instance at end of memory
            // here, return the error only if that address is actually used
            // could also be a value error if greater than 15 bit

            let (ar, br, cr) = (self.read_mem(1), self.read_mem(2), self.read_mem(3));
            let (mut a, mut b, mut c) = (0_u16, 0_u16, 0_u16);

            for (maybe_err, var) in
                [(ar, &mut a), (br, &mut b), (cr, &mut c)][..*width as usize].iter_mut()
            {
                match maybe_err {
                    Err(e) => return Err(e.clone()),
                    Ok(val) => {
                        **var = *val;
                    }
                }
            }

            // optional advance for certain jump opcodes
            let mut optional_advance = false;
            // way to stay at same address for admin commands
            let mut admin = false;

            match opname {
                OpName::Halt => (),
                OpName::Set => {
                    self.assign_reg(b)?;
                }
                OpName::Push => {
                    self.stack.push(a);
                }
                OpName::Pop => match self.stack.pop() {
                    Some(val) => {
                        self.assign_reg(val)?;
                    }
                    None => return self.err("Pop called on an empty stack.".to_string()),
                },
                OpName::Eq => {
                    let res = if b == c { 1 } else { 0 };
                    self.assign_reg(res)?;
                }
                OpName::Gt => {
                    let res = if b > c { 1 } else { 0 };
                    self.assign_reg(res)?;
                }
                OpName::Jmp => {
                    self.addr = a as usize;
                }
                OpName::Jt => {
                    if a != 0 {
                        self.addr = b as usize;
                    } else {
                        optional_advance = true;
                    }
                }
                OpName::Jf => {
                    if a == 0 {
                        self.addr = b as usize;
                    } else {
                        optional_advance = true;
                    }
                }
                OpName::Add => {
                    let res = (b + c) % (BITS_15 as u16);
                    self.assign_reg(res)?;
                }
                OpName::Mult => {
                    let res = ((b as usize) * (c as usize) % BITS_15) as u16;
                    self.assign_reg(res)?;
                }
                OpName::Mod => {
                    self.assign_reg(b % c)?;
                }
                OpName::And => {
                    self.assign_reg(b & c)?;
                }
                OpName::Or => {
                    self.assign_reg(b | c)?;
                }
                OpName::Not => {
                    let res = !b % (BITS_15 as u16);
                    self.assign_reg(res)?;
                }
                OpName::Rmem => {
                    if let Some(val) = self.memory.get(b as usize) {
                        self.assign_reg(*val)?;
                    } else {
                        return self.err(format!(
                            "Attempt to access invalid memory address {}",
                            b as usize,
                        ));
                    }
                }
                OpName::Wmem => {
                    if let Some(val) = self.memory.get_mut(a as usize) {
                        *val = b;
                    } else {
                        return self.err(format!(
                            "Attempt to access invalid memory address {}",
                            a as usize,
                        ));
                    }
                }
                OpName::Call => {
                    self.stack.push((self.addr as u16) + 2);
                    self.addr = a as usize;
                }
                OpName::Ret => match self.stack.pop() {
                    Some(val) => {
                        self.addr = val as usize;
                    }
                    None => return self.err("Ret called on an empty stack.".to_string()),
                },
                OpName::Out => match u8::try_from(a) {
                    Ok(ascii) => {
                        print!("{}", ascii as char);
                    }
                    Err(_) => {
                        return self.err(format!("Invalid ASCII code {}", a));
                    }
                },
                OpName::In => {
                    if self.input.is_empty() {
                        let mut line = String::new();
                        print!("\n> ");

                        if self.auto {
                            if let Some(command) = self.auto_commands.pop() {
                                line = command;
                                let sleep = time::Duration::from_millis(200);

                                for c in line.chars() {
                                    print!("{}", c);
                                    std::io::Write::flush(&mut std::io::stdout())?;
                                    thread::sleep(sleep);
                                }
                            } else {
                                self.auto = false;
                                std::io::Write::flush(&mut std::io::stdout())?;
                                let _ = std::io::stdin().read_line(&mut line)?;
                            }
                        } else {
                            std::io::Write::flush(&mut std::io::stdout())?;
                            let _ = std::io::stdin().read_line(&mut line)?;
                        }

                        if line == "admin\n" {
                            admin = true;
                            println!("Address: {}", self.addr);
                            println!("Registers: {:?}", self.registers);
                        } else {
                            self.input = line.bytes().map(|x| x as u16).collect();
                        }
                    }

                    if !admin {
                        let c = self.input.pop_front().unwrap();
                        self.assign_reg(c)?;
                    }
                }
                OpName::Noop => (),
            }

            if (opname.advance() || optional_advance) && !admin {
                self.addr += width + 1;
            };

            match opname {
                OpName::Halt => Ok(true),
                _ => Ok(false),
            }
        } else {
            return self.err(format!("Opcode {} is not valid.", opcode_id));
        }
    }

    pub fn run(&mut self) -> Result<bool, SynacorErr> {
        loop {
            let step = self.step();
            match step {
                Err(_) | Ok(true) => return step,
                Ok(false) => (),
            }
        }
    }
}

const SOLUTION: &'static [&'static str] = &[
    "take tablet\n",
    "use tablet\n",
    "doorway\n",
    "north\n",
    "north\n",
    "bridge\n",
    "continue\n",
    "down\n",
    "east\n",
    "take empty lantern\n",
    "west\n",
    "west\n",
    "passage\n",
    "ladder\n",
    "west\n",
    "north\n",
    "south\n",
    "north\n",
    "take can\n",
    "use can\n",
    "use lantern\n",
    "west\n",
    "ladder\n",
    "darkness\n",
    "continue\n",
    "west\n",
    "west\n",
    "west\n",
    "west\n",
    "north\n",
    "take red coin\n",
    "north\n",
    "east\n",
    "take concave coin\n",
    "down\n",
    "take corroded coin\n",
    "up\n",
    "west\n",
    "west\n",
    "take blue coin\n",
    "up\n",
    "take shiny coin\n",
    "down\n",
    "east\n",
    "use blue coin\n",
    "use red coin\n",
    "use shiny coin\n",
    "use concave coin\n",
    "use corroded coin\n",
    "north\n",
    "take teleporter\n",
    "use teleporter\n",
];
