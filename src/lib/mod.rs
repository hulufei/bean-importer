mod rules;

use self::rules::Rules;
use anyhow;
use fehler::throws;

type Error = anyhow::Error;

#[derive(Clone)]
pub enum Flow {
    Income,
    Expense,
    Unknown(String),
}

impl Flow {
    pub fn is_unknown(&self) -> bool {
        match self {
            Flow::Unknown(_) => true,
            _ => false,
        }
    }
}

impl From<&str> for Flow {
    fn from(s: &str) -> Self {
        match s {
            "收入" => Flow::Income,
            "支出" => Flow::Expense,
            _ => Flow::Unknown(s.to_owned()),
        }
    }
}

impl Default for Flow {
    fn default() -> Self {
        Flow::Unknown("default".to_owned())
    }
}

pub trait Transaction {
    #[throws]
    fn date(&self) -> String;

    #[throws]
    fn payee(&self) -> String;

    #[throws]
    fn narration(&self) -> String;

    /// Keys must begin with a lowercase character from a-z and may contain (uppercase or lowercase) letters,
    /// numbers, dashes and underscores.
    #[throws]
    fn metadata(&self) -> Vec<(String, String)> {
        vec![]
    }

    #[throws]
    fn amount(&self) -> f32;

    #[throws]
    fn flow(&self) -> Flow;
}

pub struct Bean {
    transactions: Vec<Box<dyn Transaction>>,
    fund_account: String,
}

impl Bean {
    pub fn new(account: &str) -> Self {
        Self {
            transactions: Vec::new(),
            fund_account: account.to_owned(),
        }
    }

    pub fn add(&mut self, transaction: impl Transaction + 'static) {
        self.transactions.push(Box::new(transaction));
    }

    #[throws]
    pub fn output_with_rules(&self, rules: Rules) -> String {
        let mut output = String::new();
        for transaction in &self.transactions {
            let payee = transaction.payee()?;
            let flow = transaction.flow()?;
            let mut to_account = rules.get_payee_account(&payee).unwrap_or("").to_owned();

            let flag = if to_account.is_empty() || flow.is_unknown() {
                "!"
            } else {
                "*"
            };

            let mut metadata = transaction
                .metadata()?
                .iter()
                .map(|(k, v)| format!(r#"{}: "{}""#, k, v))
                .collect::<Vec<_>>()
                .join("\n");

            if !metadata.is_empty() {
                metadata.insert_str(0, "\n  ");
            }

            if !to_account.is_empty() {
                to_account.push(' ');
            }

            output.push_str(
                format!(
                    r##"{date} {flag} "{payee}" "{narration}"{metadata}
  {account}{amount} CNY
  {fund_account}"##,
                    date = transaction.date()?,
                    payee = payee,
                    narration = transaction.narration()?,
                    flag = flag,
                    account = to_account,
                    amount = transaction.amount()?,
                    metadata = metadata,
                    fund_account = self.fund_account
                )
                .as_str(),
            );
            output.push('\n');
        }
        output
    }

    #[throws]
    pub fn output(&self) -> String {
        let mut rules = Rules::from_file()?;
        rules.merge_with_edit(&self.transactions)?;
        self.output_with_rules(rules)?
    }
}

#[cfg(test)]
mod tests {
    use super::Bean;
    use super::Rules;
    use crate::test_helpers::MockTransanction;
    use anyhow;
    use fehler::throws;

    type Error = anyhow::Error;

    #[test]
    #[throws]
    fn test_output() {
        let mut bean = Bean::new("Assets:Test");
        let mut transaction = MockTransanction::default();
        transaction.date = "2020-04-01";
        transaction.payee = "SomeShop";
        transaction.narration = "some notes";
        bean.add(transaction);
        assert_eq!(
            bean.output_with_rules(Rules::from_str("")?)?,
            r#"2020-04-01 ! "SomeShop" "some notes"
  0 CNY
  Assets:Test
"#
        );
    }
}