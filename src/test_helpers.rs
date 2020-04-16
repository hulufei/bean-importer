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
    pub flow: Flow<'a>,
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
    fn date(&self) -> &str {
        self.date
    }
    #[throws]
    fn payee(&self) -> &str {
        self.payee
    }
    #[throws]
    fn narration(&self) -> &str {
        self.narration
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
    fn metadata(&self) -> Vec<(&str, &str)> {
        self.meta.clone()
    }
}
