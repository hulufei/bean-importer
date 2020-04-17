mod rules;

use self::rules::Rules;
use anyhow;
use fehler::throws;

type Error = anyhow::Error;

#[derive(Clone)]
pub enum Flow<'a> {
    Income,
    Expense,
    Unknown(&'a str),
}

impl Flow<'_> {
    pub fn is_unknown(&self) -> bool {
        match self {
            Flow::Unknown(_) => true,
            _ => false,
        }
    }
}

impl<'a> From<&'a str> for Flow<'a> {
    fn from(s: &'a str) -> Self {
        match s {
            "收入" => Flow::Income,
            "支出" => Flow::Expense,
            _ => Flow::Unknown(s),
        }
    }
}

impl Default for Flow<'_> {
    fn default() -> Self {
        Flow::Unknown("default")
    }
}

pub trait Transaction {
    #[throws]
    fn date(&self) -> &str;

    #[throws]
    fn payee(&self) -> &str;

    #[throws]
    fn fund(&self) -> &str {
        ""
    }

    #[throws]
    fn narration(&self) -> &str;

    /// Keys must begin with a lowercase character from a-z and may contain (uppercase or lowercase) letters,
    /// numbers, dashes and underscores.
    #[throws]
    fn metadata(&self) -> Vec<(&str, &str)> {
        vec![]
    }

    #[throws]
    fn amount(&self) -> f32;

    #[throws]
    fn flow(&self) -> Flow;
}

pub struct Bean<'a> {
    transactions: Vec<Box<dyn Transaction>>,
    default_fund: &'a str,
}

impl<'a> Bean<'a> {
    pub fn new(default_fund: &'a str) -> Self {
        Self {
            transactions: Vec::new(),
            default_fund,
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
            let fund = transaction.fund()?;
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
                    payee = rules.get_payee_alias(payee).unwrap_or(payee),
                    narration = transaction.narration()?,
                    flag = flag,
                    account = to_account,
                    amount = transaction.amount()?,
                    metadata = metadata,
                    fund_account = rules.get_fund_account(fund).unwrap_or(self.default_fund)
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
        let rules = r#"
[payee]
"SomeShop" = "Expenses:Custom"
"#;
        assert_eq!(
            bean.output_with_rules(Rules::from_str(rules)?)?,
            r#"2020-04-01 ! "SomeShop" "some notes"
  Expenses:Custom 0 CNY
  Assets:Test
"#
        );
    }

    #[test]
    #[throws]
    fn test_output_with_fund() {
        let mut bean = Bean::new("Assets:Test");
        let mut transaction = MockTransanction::default();
        transaction.fund = "custom";
        bean.add(transaction);
        let rules = r#"
[fund]
"custom" = "Assets:Custom"
"#;
        assert_eq!(
            bean.output_with_rules(Rules::from_str(rules)?)?,
            r#" ! "" ""
  0 CNY
  Assets:Custom
"#
        );
    }

    #[throws]
    #[test]
    fn test_alias() {
        let mut bean = Bean::new("Assets:Test");
        let mut transaction = MockTransanction::default();
        transaction.payee = "test";
        bean.add(transaction);
        let rules = r#"
[payee]
"test" = { alias = "aliased", account = "Expenses:Aliased" }
"#;
        assert_eq!(
            bean.output_with_rules(Rules::from_str(rules)?)?,
            r#" ! "aliased" ""
  Expenses:Aliased 0 CNY
  Assets:Test
"#
        );
    }
}
