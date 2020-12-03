use anyhow::Result;
use std::fs::File as StdFile;
use std::io::{self, Stdout as StdStdout, Write};

enum WriterDestination {
    File(StdFile),
    Stdout(StdStdout),
}

pub struct DumpWriter {
    writer_destination: WriterDestination,
}

impl DumpWriter {
    pub fn new(target: Option<String>) -> Result<Self> {
        let writer_destination = match target {
            Some(t) => WriterDestination::File(Self::create_target_file(t)?),
            None => WriterDestination::Stdout(io::stdout()),
        };

        Ok(Self { writer_destination })
    }

    pub fn write_all(&mut self, data: &[u8]) -> Result<()> {
        match self.writer_destination {
            WriterDestination::File(ref mut f) => f.write_all(data).map_err(|e| e.into()),
            WriterDestination::Stdout(ref mut s) => s.write_all(data).map_err(|e| e.into()),
        }
    }

    pub fn can_log_to_stdout(&self) -> bool {
        matches!(self.writer_destination, WriterDestination::File(_))
    }

    fn create_target_file(destination: String) -> Result<StdFile> {
        StdFile::create(&destination).map_err(|e| e.into())
    }
}
