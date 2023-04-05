use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::emit;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

#[derive(Debug, Clone)]
pub enum Location {
    Address(usize),
    Code {
        start: usize,
        end: usize,
        file: PathBuf,
        code: String,
    },
    IO,
}

#[derive(Debug, Clone)]
pub struct SynacorErr {
    pub location: Location,
    pub details: String,
}

impl SynacorErr {
    pub fn new_addr(addr: usize, details: String) -> Self {
        Self {
            location: Location::Address(addr),
            details,
        }
    }

    pub fn new_code(
        start: usize,
        end: usize,
        file: PathBuf,
        code: String,
        details: String,
    ) -> Self {
        Self {
            location: Location::Code {
                start,
                end,
                file,
                code,
            },
            details,
        }
    }

    pub fn emit(&self) -> Result<(), codespan_reporting::files::Error> {
        match &self.location {
            Location::Code {
                start,
                end,
                file,
                code,
            } => {
                let mut files = SimpleFiles::new();

                let name = file.to_string_lossy();
                let file_id = files.add(name, code);

                let diagnostic = Diagnostic::error()
                    .with_message(&self.details)
                    .with_labels(vec![Label::primary(file_id, *start..*end)]);

                let writer = StandardStream::stderr(ColorChoice::Always);
                let config = codespan_reporting::term::Config::default();

                emit(&mut writer.lock(), &config, &files, &diagnostic)?;
            }
            _ => (),
        }
        Ok(())
    }
}

impl fmt::Display for SynacorErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.location {
            Location::Address(addr) => {
                write!(f, "Failure at address {}.\n\n{}", addr, self.details)
            }
            Location::Code { .. } => write!(f, "{}", self.details),
            Location::IO => write!(f, "IO error: {}", self.details),
        }
    }
}

impl Error for SynacorErr {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<std::io::Error> for SynacorErr {
    fn from(e: std::io::Error) -> Self {
        SynacorErr {
            location: Location::IO,
            details: format!("{}", e),
        }
    }
}
