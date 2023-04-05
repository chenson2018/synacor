use clap::Parser;
use main_error::MainError;

use synacor::cli::{Cli, Command, FileType};
use synacor::convert::{asm_to_u16, bin_to_u16, u16_to_asm, u16_to_bin};
use synacor::vm::VM;

fn main() -> Result<(), MainError> {
    let args = Cli::parse();

    // can't use ? here because of codespan-reporting
    let read_memory = match args.ftype {
        FileType::Binary => bin_to_u16(&args.path),
        FileType::Assembly => asm_to_u16(&args.path),
    };

    match read_memory {
        Err(e) => e.emit()?,
        Ok(memory) => match (args.command, args.ftype) {
            (Command::Run { auto }, _) => {
                let mut vm = VM::new(memory, auto);
                vm.run()?;
            }
            (Command::Convert { out_path }, FileType::Binary) => u16_to_asm(memory, &out_path)?,
            (Command::Convert { out_path }, FileType::Assembly) => {
                match u16_to_bin(memory, &out_path) {
                    Err(e) => e.emit()?,
                    _ => (),
                }
            }
        },
    }

    Ok(())
}
