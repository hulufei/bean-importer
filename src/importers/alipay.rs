use super::csv::{pick, Parser};
use crate::lib::{Bean, Flow, Transaction};
use anyhow;
use csv::StringRecord;
use fehler::throws;
use std::path::PathBuf;

pub struct Alipay(StringRecord);

type Error = anyhow::Error;

impl<'a> Alipay {
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

impl Transaction for Alipay {
    #[throws]
    fn date(&self) -> String {
        self.pick("date", 3, |s| s.split_whitespace().next().map(String::from))?
    }

    #[throws]
    fn payee(&self) -> String {
        self.pick("payee", 7, Self::default_transform)?
    }

    #[throws]
    fn narration(&self) -> String {
        self.pick("narration", 8, Self::default_transform)?
    }

    #[throws]
    fn amount(&self) -> f32 {
        let amount: f32 = self.pick("amount", 9, |s| s.parse().ok())?;
        match self.flow()? {
            Flow::Income => -amount,
            _ => amount,
        }
    }

    #[throws]
    fn flow(&self) -> Flow {
        let flow = self.pick("flow", 10, Self::default_transform)?;
        Flow::from(flow.as_str())
    }
}

#[throws]
pub fn import(input: PathBuf) -> String {
    let parser = Parser::new(input, 4);
    let bean = Bean::new("Assets:Alipay");
    parser.output(bean, Alipay::new)?
}

#[cfg(test)]
mod tests {
    use super::Alipay;
    use crate::lib::Transaction;
    use crate::test_helpers::gen_record;
    use fehler::throws;

    #[derive(Default)]
    pub struct Trans<'a> {
        create_date: &'a str,
        pay_date: &'a str,
        modify_date: &'a str,
        trade_type: &'a str,
        trade_source: &'a str,
        payee: &'a str,
        commodity: &'a str,
        flow: &'a str,
        amount: &'a str,
        fee: &'a str,
        refund: &'a str,
        fund_status: &'a str,
        status: &'a str,
        trade_id: &'a str,
        store_id: &'a str,
        remark: &'a str,
    }
    impl<'a> Trans<'a> {
        pub fn as_string(&self) -> String {
            vec![
                self.trade_id,
                self.store_id,
                self.create_date,
                self.pay_date,
                self.modify_date,
                self.trade_source,
                self.trade_type,
                self.payee,
                self.commodity,
                self.amount,
                self.flow,
                self.status,
                self.fee,
                self.refund,
                self.remark,
                self.fund_status,
            ]
            .join(",")
        }
    }

    type Error = anyhow::Error;

    #[test]
    #[throws]
    fn get_date() {
        let t = Trans {
            pay_date: "2020-03-30 18:46:56",
            ..Trans::default()
        };
        let r = gen_record(&t.as_string())?;
        let transaction = Alipay::new(r);
        assert_eq!(transaction.date()?, "2020-03-30")
    }
}
