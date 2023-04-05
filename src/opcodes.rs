use bimap::BiHashMap;
use std::collections::HashMap;

#[derive(Debug)]
pub enum OpName {
    Halt,
    Set,
    Push,
    Pop,
    Eq,
    Gt,
    Jmp,
    Jt,
    Jf,
    Add,
    Mult,
    Mod,
    And,
    Or,
    Not,
    Rmem,
    Wmem,
    Call,
    Ret,
    Out,
    In,
    Noop,
}

impl OpName {
    pub fn advance(&self) -> bool {
        match self {
            Self::Halt | Self::Jmp | Self::Jt | Self::Jf | Self::Ret | Self::Call => false,
            _ => true,
        }
    }
}

impl TryFrom<u16> for OpName {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Halt),
            1 => Ok(Self::Set),
            2 => Ok(Self::Push),
            3 => Ok(Self::Pop),
            4 => Ok(Self::Eq),
            5 => Ok(Self::Gt),
            6 => Ok(Self::Jmp),
            7 => Ok(Self::Jt),
            8 => Ok(Self::Jf),
            9 => Ok(Self::Add),
            10 => Ok(Self::Mult),
            11 => Ok(Self::Mod),
            12 => Ok(Self::And),
            13 => Ok(Self::Or),
            14 => Ok(Self::Not),
            15 => Ok(Self::Rmem),
            16 => Ok(Self::Wmem),
            17 => Ok(Self::Call),
            18 => Ok(Self::Ret),
            19 => Ok(Self::Out),
            20 => Ok(Self::In),
            21 => Ok(Self::Noop),
            _ => Err(format!("Opcode {} is invalid", value)),
        }
    }
}

lazy_static! {
    pub static ref ASM_CONVERT: BiHashMap<u16, &'static str> =
        {
            BiHashMap::from_iter(
                [
                // opcodes
                (0, "halt"),
                (1,"set"),
                (2,"push"),
                (3,"pop"),
                (4,"eq"),
                (5,"gt"),
                (6,"jmp"),
                (7,"jt"),
                (8,"jf"),
                (9,"add"),
                (10,"mult"),
                (11,"mod"),
                (12,"and"),
                (13,"or"),
                (14,"not"),
                (15,"rmem"),
                (16,"wmem"),
                (17,"call"),
                (18,"ret"),
                (19,"out"),
                (20,"in"),
                (21,"noop"),
                // registers
                (32768,"$0"),
                (32769,"$1"),
                (32770,"$2"),
                (32771,"$3"),
                (32772,"$4"),
                (32773,"$5"),
                (32774,"$6"),
                (32775,"$7"),
                ])
        };

    pub static ref INS_WIDTH: HashMap<u16, usize> = {
        HashMap::from_iter([
            (0, 0),
            (1, 2),
            (2, 1),
            (3, 1),
            (4, 3),
            (5, 3),
            (6, 1),
            (7, 2),
            (8, 2),
            (9, 3),
            (10, 3),
            (11, 3),
            (12, 3),
            (13, 3),
            (14, 2),
            (15, 2),
            (16, 2),
            (17, 1),
            (18, 0),
            (19, 1),
            (20, 1),
            (21, 0),
        ])
    };

}
