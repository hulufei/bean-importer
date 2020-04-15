pub use super::csv::{pick, Parser};
use crate::lib::{Bean, Flow, Transaction};
use anyhow;
use csv::StringRecord;
use fehler::throws;
use std::path::PathBuf;

pub struct Wechat(StringRecord);

type Error = anyhow::Error;

impl<'a> Wechat {
    pub fn new(record: StringRecord) -> Self {
        Self(record)
    }

    #[throws]
    fn pick<F, T>(&'a self, name: &str, i: usize, transform: F) -> T
    where
        F: Fn(&'a str) -> Option<T>,
    {
        pick(&self.0, name, i, transform)?
    }

    fn default_transform(s: &str) -> Option<String> {
        Some(s.to_owned())
    }
}

impl Transaction for Wechat {
    #[throws]
    fn date(&self) -> String {
        self.pick("date", 0, |s| s.split_whitespace().next().map(String::from))?
    }

    #[throws]
    fn payee(&self) -> String {
        self.pick("payee", 2, Self::default_transform)?
    }

    #[throws]
    fn narration(&self) -> String {
        self.pick("narration", 3, Self::default_transform)?
    }

    #[throws]
    fn amount(&self) -> f32 {
        let amount: f32 = self.pick("amount", 5, |s| s.trim_start_matches('¥').parse().ok())?;
        match self.flow()? {
            Flow::Income => -amount,
            _ => amount,
        }
    }

    #[throws]
    fn flow(&self) -> Flow {
        let flow = self.pick("flow", 4, Self::default_transform)?;
        Flow::from(flow.as_str())
    }

    #[throws]
    fn metadata(&self) -> Vec<(String, String)> {
        let mut meta = vec![];
        if let Flow::Unknown(s) = self.flow()? {
            meta.push(("unknown_flow".to_owned(), s))
        }
        meta
    }
}

#[throws]
pub fn import(input: PathBuf) -> String {
    let parser = Parser::new(input, 16);
    let bean = Bean::new("Assets:Wechat");
    parser.output(bean, Wechat::new)?
}

#[cfg(test)]
mod tests {
    use super::Wechat;
    use crate::lib::Transaction;
    use crate::test_helpers::gen_record;
    use fehler::throws;

    #[derive(Default)]
    pub struct Trans<'a> {
        date: &'a str,
        trade_type: &'a str,
        payee: &'a str,
        commodity: &'a str,
        flow: &'a str,
        amount: &'a str,
        payment: &'a str,
        status: &'a str,
        trade_id: &'a str,
        store_id: &'a str,
        remark: &'a str,
    }
    impl<'a> Trans<'a> {
        pub fn as_string(&self) -> String {
            vec![
                self.date,
                self.trade_type,
                self.payee,
                self.commodity,
                self.flow,
                self.amount,
                self.payment,
                self.status,
                self.trade_id,
                self.store_id,
                self.remark,
            ]
            .join(",")
        }
    }

    type Error = anyhow::Error;

    #[test]
    #[throws]
    fn get_date() {
        let t = Trans {
            date: "2020-03-30 18:46:56",
            ..Trans::default()
        };
        let r = gen_record(&t.as_string())?;
        let wechat = Wechat::new(r);
        assert_eq!(wechat.date()?, "2020-03-30")
    }

    #[test]
    #[throws]
    fn add_unknown_flow_to_metadata() {
        let t = Trans {
            flow: "unknownflow",
            ..Trans::default()
        };
        let r = gen_record(&t.as_string())?;
        let wechat = Wechat::new(r);
        assert_eq!(
            wechat.metadata()?,
            vec![("unknown_flow".to_owned(), t.flow.to_owned())]
        )
    }
}
