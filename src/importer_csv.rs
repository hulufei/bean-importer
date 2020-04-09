use crate::common::{handle_new_rules, load_rules, Alipay, Bean, Trasaction, Wechat};
use csv::{ReaderBuilder, StringRecord, Trim};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

struct CsvParser {
    // start from 0
    header_line: usize,
    payee_index: usize,
    input: PathBuf,
}

impl CsvParser {
    fn new(input: PathBuf, header_line: usize, payee_index: usize) -> Self {
        Self {
            input,
            header_line,
            payee_index,
        }
    }

    fn parse(&self) -> Result<Vec<StringRecord>, Box<dyn Error>> {
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
        let records: Vec<StringRecord> = rdr.records().filter_map(|result| result.ok()).collect();
        let (rules_file, new_rules) = self.gen_rules(&records)?;
        handle_new_rules(rules_file, new_rules)?;
        Ok(records)
    }

    fn gen_rules(
        &self,
        records: &[StringRecord],
    ) -> Result<(File, HashMap<String, &str>), Box<dyn Error>> {
        // Rules file
        let (file, rules) = load_rules()?;
        // CSV file
        let mut new_rules = HashMap::new();
        for record in records {
            let payee = record.get(self.payee_index).unwrap();
            if rules.get(payee).is_none() {
                new_rules.insert(payee.to_owned(), "");
            }
        }
        Ok((file, new_rules))
    }
}

pub fn import_alipay(path: PathBuf) -> Result<Bean, Box<dyn Error>> {
    let alipay = CsvParser::new(path, 4, 7);
    let records = alipay.parse()?;
    let mut bean = Bean::new("Assets:Alipay");
    for record in records {
        bean.add(Trasaction::from(Alipay::new(record)));
    }
    Ok(bean)
}

pub fn import_wechat(path: PathBuf) -> Result<Bean, Box<dyn Error>> {
    let wechat = CsvParser::new(path, 16, 2);
    let records = wechat.parse()?;
    let mut bean = Bean::new("Assets:Wechat");
    for record in records {
        bean.add(Trasaction::from(Wechat::new(record)));
    }
    Ok(bean)
}
