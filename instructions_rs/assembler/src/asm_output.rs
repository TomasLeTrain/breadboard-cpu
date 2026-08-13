use miette::{IntoDiagnostic, Result, miette};
use std::{
    fs::{File, OpenOptions},
    io::Write,
};

pub trait AsmOutput {
    fn generate_output(&mut self, asm: Vec<u8>) -> Result<()>;
}

pub struct LogisimOutput {
    file: File,
}

impl LogisimOutput {
    pub fn new(file_name: &str) -> Result<Self> {
        Ok(LogisimOutput {
            file: OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(file_name)
                .into_diagnostic()?,
        })
    }
}

impl AsmOutput for LogisimOutput {
    fn generate_output(&mut self, asm: Vec<u8>) -> Result<()> {
        self.file
            .write_all(b"v3.0 hex words plain\n")
            .into_diagnostic()?;

        for (i, curr) in asm.iter().enumerate() {
            write!(self.file, "{:02x}", curr).into_diagnostic()?;

            if i % 16 == 15 {
                writeln!(self.file).into_diagnostic()?;
            } else {
                write!(self.file, " ").into_diagnostic()?;
            }
        }
        Ok(())
    }
}

pub struct BinaryOutput {
    file: File,
    file_size: usize,
}

impl BinaryOutput {
    pub fn new(file_name: &str, file_size: usize) -> Result<Self> {
        Ok(Self {
            file: OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(file_name)
                .into_diagnostic()?,
            file_size,
        })
    }
}

impl AsmOutput for BinaryOutput {
    fn generate_output(&mut self, asm: Vec<u8>) -> Result<()> {
        if asm.len() > self.file_size {
            Err(miette!("generated asm larger than target file size"))?;
        }

        self.file.write_all(&asm).into_diagnostic()?;

        let pad_len = self.file_size - asm.len();

        for _ in 0..pad_len {
            self.file.write(&[0]).into_diagnostic()?;
        }

        Ok(())
    }
}
