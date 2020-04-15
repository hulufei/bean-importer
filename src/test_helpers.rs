use crate::lib::{Flow, Transaction};
use csv::{ReaderBuilder, StringRecord, Trim};
use fehler::throws;

type Error = anyhow::Error;

#[throws]
pub fn gen_record(s: &str) -> StringRecord {
    let mut record = StringRecord::new();
    ReaderBuilder::new()
        .trim(Trim::All)
        .has_headers(false)
        .from_reader(s.as_bytes())
        .read_record(&mut record)?;
    record
}

#[derive(Default)]
pub struct MockTransanction<'a> {
    pub date: &'a str,
    pub payee: &'a str,
    pub narration: &'a str,
    pub flow: Flow,
    pub amount: f32,
    pub meta: Vec<(&'a str, &'a str)>,
}

impl MockTransanction<'_> {
    pub fn gen_with_payee(payee: &'static str) -> Box<dyn Transaction> {
        Box::new(MockTransanction {
            payee,
            ..MockTransanction::default()
        })
    }
}

impl Transaction for MockTransanction<'_> {
    #[throws]
    fn date(&self) -> String {
        self.date.to_owned()
    }
    #[throws]
    fn payee(&self) -> String {
        self.payee.to_owned()
    }
    #[throws]
    fn narration(&self) -> String {
        self.narration.to_owned()
    }
    #[throws]
    fn flow(&self) -> Flow {
        self.flow.clone()
    }
    #[throws]
    fn amount(&self) -> f32 {
        self.amount
    }
    #[throws]
    fn metadata(&self) -> Vec<(String, String)> {
        self.meta
            .iter()
            .map(|v| (v.0.to_owned(), v.1.to_owned()))
            .collect()
    }
}
