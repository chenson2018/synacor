use itertools::Itertools;
use std::fs::File;
use std::io::Write;
use std::num::ParseIntError;
use std::path::PathBuf;

use crate::error::SynacorErr;
use crate::opcodes::{ASM_CONVERT, INS_WIDTH};

pub fn bin_to_u16(path: &PathBuf) -> Result<Vec<u16>, SynacorErr> {
    let bytes = std::fs::read(path)?;

    Ok(bytes
        .iter()
        .tuples()
        .map(|(low, high)| u16::from_le_bytes([*low, *high]))
        .collect())
}

pub fn u16_to_bin(memory: Vec<u16>, out_path: &PathBuf) -> Result<(), SynacorErr> {
    let mut file = File::create(out_path)?;
    let le: Vec<u8> = memory.iter().flat_map(|x| u16::to_le_bytes(*x)).collect();
    file.write_all(&le)?;
    println!("Created binary file {}", out_path.display());
    Ok(())
}

// Line addresses are optional!

pub fn asm_to_u16(path: &PathBuf) -> Result<Vec<u16>, SynacorErr> {
    let asm = std::fs::read_to_string(path)?;

    asm.lines()
        .enumerate()
        .flat_map(|(line_num, line)| {
            let address_strip = match line.split_once(":") {
                Some((_, tail)) => tail,
                None => line,
            };

            address_strip
                .split_whitespace()
                .collect::<Vec<&str>>()
                .iter()
                .filter(|lexeme| lexeme != &&"data")
                .enumerate()
                .map(|(line_idx, lexeme)| {
                    let lookup_or_parse: Result<u16, ParseIntError> =
                        match ASM_CONVERT.get_by_right(lexeme) {
                            Some(int) => Ok(*int),
                            None => {
                                let without_prefix = lexeme.trim_start_matches("0x");
                                u16::from_str_radix(without_prefix, 16)
                            }
                        };

                    match lookup_or_parse {
                        Ok(bin) => {
                            if bin >= 32776 {
                                let (start, end) = char_range(&asm, line_num, line_idx);
                                Err(SynacorErr::new_code(
                                    start,
                                    end,
                                    path.to_path_buf(),
                                    asm.to_string(),
                                    format!("Value {} is invalid.", bin),
                                ))
                            } else {
                                Ok(bin)
                            }
                        }
                        Err(_) => {
                            let (start, end) = char_range(&asm, line_num, line_idx);
                            Err(SynacorErr::new_code(
                                start,
                                end,
                                path.to_path_buf(),
                                asm.to_string(),
                                format!("\"{}\" does not parse as a valid u16.", lexeme),
                            ))
                        }
                    }
                })
                .collect::<Vec<Result<u16, SynacorErr>>>()
        })
        .collect()
}

// TODO check this... not working in challenge.asm in middle, maybe because of data filter or
// address numbers

fn char_range(asm: &String, idx: usize, line_idx: usize) -> (usize, usize) {
    let splits: Vec<Vec<&str>> = asm
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<&str>>())
        .collect();
    let mem_before = &splits[idx][..line_idx];
    let chars_before = mem_before.len() + mem_before.iter().fold(0, |acc, x| acc + x.len());
    let current = &splits[idx][line_idx].len();
    (chars_before, chars_before + current)
}

pub fn u16_to_asm(memory: Vec<u16>, out_path: &PathBuf) -> Result<(), SynacorErr> {
    let mut res: Vec<Result<String, SynacorErr>> = Vec::new();
    let mut it = memory.iter().enumerate();

    while let Some((addr, val)) = it.next() {
        let line: Result<String, SynacorErr> = if let (Some(opcode_str), Some(width)) =
            (ASM_CONVERT.get_by_left(val), INS_WIDTH.get(val))
        {
            let lexemes: Vec<(usize, u16)> = it
                .by_ref()
                .take(*width)
                .map(|(addr, val)| (addr, *val))
                .collect();

            let res_operands: Result<Vec<String>, SynacorErr> = lexemes
                .iter()
                .map(|(addr, val)| {
                    if val >= &32768 {
                        if let Some(register) = ASM_CONVERT.get_by_left(val) {
                            Ok(register.to_string())
                        } else {
                            Err(SynacorErr::new_addr(
                                *addr,
                                format!("Invalid value {}.", val),
                            ))
                        }
                    } else {
                        Ok(format!("{:#06x}", val))
                    }
                })
                .collect();

            match res_operands {
                Err(e) => Err(e),
                Ok(operands) => Ok(
                    vec![vec![format!("{:#06x}:{}", addr, opcode_str)], operands]
                        .iter()
                        .flatten()
                        .join(" "),
                ),
            }
        } else {
            Ok(format!("{:#06x}:data {:#06x}", addr, val))
        };
        res.push(line);
    }

    // report first Err

    if let Some(Err(e)) = res.iter().find(|line| line.is_err()) {
        Err(e.clone())
    } else {
        let full_asm = res
            .into_iter()
            .flatten()
            .collect::<Vec<String>>()
            .join("\n");
        let mut file = File::create(out_path)?;
        write!(file, "{}", full_asm)?;
        println!("Created assembly file {}", out_path.display());
        Ok(())
    }
}
