#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use synacor::convert::{asm_to_u16, bin_to_u16, u16_to_asm, u16_to_bin};
    use synacor::error::SynacorErr;

    #[test]
    fn example_asm() -> Result<(), SynacorErr> {
        let original_asm = PathBuf::from("examples/ex.asm");
        let converted_bin = PathBuf::from("ex.bin");
        let converted_asm = PathBuf::from("roundtrip.asm");

        // convert the example asm to Vec<u16>
        let memory_original = asm_to_u16(&original_asm)?;

        // convert Vec<u16> to binary and read into memory
        u16_to_bin(memory_original.clone(), &converted_bin)?;
        let memory_converted = bin_to_u16(&converted_bin)?;

        // check that the conversion keeps the same memory
        assert_eq!(memory_original, memory_converted);

        // reconvert back to assembly and read into memory
        u16_to_asm(memory_converted, &converted_asm)?;
        let roundtrip_memory = asm_to_u16(&converted_asm)?;

        // check that we still match the original
        assert_eq!(memory_original, roundtrip_memory);

        // clean up files
        std::fs::remove_file(&converted_bin)?;
        std::fs::remove_file(&converted_asm)?;

        Ok(())
    }

    #[test]
    fn challenge_binary() -> Result<(), SynacorErr> {
        let original_bin = PathBuf::from("examples/challenge.bin");
        let converted_asm = PathBuf::from("challenge.asm");
        let converted_bin = PathBuf::from("roundtrip.bin");

        // convert the challenge binary to Vec<u16>
        let memory_original = bin_to_u16(&original_bin)?;

        // convert Vec<u16> to assembly and read into memory
        u16_to_asm(memory_original.clone(), &converted_asm)?;
        let memory_converted = asm_to_u16(&converted_asm)?;

        // check that the conversion keeps the same memory
        assert_eq!(memory_original, memory_converted);

        // reconvert back to binary and read into memory
        u16_to_bin(memory_converted, &converted_bin)?;
        let roundtrip_memory = bin_to_u16(&converted_bin)?;

        // check that we still match the original
        assert_eq!(memory_original, roundtrip_memory);

        // clean up files
        std::fs::remove_file(&converted_bin)?;
        std::fs::remove_file(&converted_asm)?;

        Ok(())
    }
}
