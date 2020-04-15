use crate::lib::{Bean, Transaction};
use anyhow::anyhow;
use csv::{ReaderBuilder, StringRecord, Trim};
use fehler::throws;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

type Error = anyhow::Error;

#[throws]
pub fn pick<'a, F, T>(record: &'a StringRecord, name: &str, i: usize, transform: F) -> T
where
    F: Fn(&'a str) -> Option<T>,
{
    record
        .get(i)
        .and_then(transform)
        .ok_or_else(|| anyhow!("Can't get {} from {:?} with index {}", name, record, i))?
}

pub struct Parser {
    // start from 0
    header_line: usize,
    input: PathBuf,
}

impl Parser {
    pub fn new(input: PathBuf, header_line: usize) -> Self {
        Self { input, header_line }
    }

    #[throws]
    pub fn parse(&self) -> Vec<StringRecord> {
        let mut file = File::open(&self.input)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        contents = contents
            .lines()
            .skip(self.header_line)
            .collect::<Vec<_>>()
            .join("\n");
        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(contents.as_bytes());
        rdr.records().filter_map(|result| result.ok()).collect()
    }

    #[throws]
    pub fn output<F, T: 'static + Transaction>(&self, mut bean: Bean, constructor: F) -> String
    where
        F: Fn(StringRecord) -> T,
    {
        for record in self.parse()? {
            bean.add(constructor(record));
        }
        bean.output()?
    }
}
