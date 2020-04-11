use anyhow::{anyhow, Context};
use fehler::{throw, throws};
use std::collections::HashMap;
use std::env::var;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::{self, Write};
use std::process;
use toml::Value;

type Error = anyhow::Error;

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

pub trait Transaction {
    #[throws]
    fn date(&self) -> String;

    #[throws]
    fn payee(&self) -> String;

    #[throws]
    fn narration(&self) -> String;

    #[throws]
    fn metadata(&self) -> Vec<(String, String)> {
        vec![]
    }

    #[throws]
    fn amount(&self) -> f32;

    #[throws]
    fn flow(&self) -> Flow;
}

static RULES_PATH: &str = "rules.toml";

pub struct Bean {
    transactions: Vec<Box<dyn Transaction>>,
    fund_account: String,
    rules: Option<Value>,
}

impl Bean {
    pub fn new(account: &str) -> Self {
        Self {
            transactions: Vec::new(),
            fund_account: account.to_owned(),
            rules: None,
        }
    }

    pub fn add(&mut self, transaction: impl Transaction + 'static) {
        self.transactions.push(Box::new(transaction));
    }

    #[throws]
    pub fn load_rules() -> Value {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(RULES_PATH)
            .context("Load rules failed")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        contents.parse::<Value>()?
    }

    #[throws]
    pub fn set_rules(&mut self) -> &mut Self {
        let rules = Self::load_rules()?;
        let mut new_rules = HashMap::new();
        for transaction in &self.transactions {
            let payee = transaction.payee()?;
            if rules.get(&payee).is_none() {
                new_rules.insert(payee, "");
            }
        }
        let has_empty = new_rules.values().any(|v| v.is_empty());
        if has_empty {
            print!("There are new rules should be specified first, save and edit? (yes/no)");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            match input.trim() {
                "yes" | "y" => {
                    // Append new rules
                    let contents = new_rules
                        .iter()
                        .map(|(k, v)| format!("'{}' = '{}'", k, v))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let mut rules_file = OpenOptions::new().append(true).open(RULES_PATH)?;
                    rules_file.write_all(b"\n")?;
                    rules_file.write_all(contents.as_bytes())?;
                    // edit
                    let editor = var("EDITOR").unwrap();
                    process::Command::new(editor).arg(RULES_PATH).status()?;
                    // Reload rules
                    self.rules = Some(Self::load_rules()?);
                }
                _ => throw!(anyhow!("Exit")),
            }
        } else {
            self.rules = Some(rules);
        }
        self
    }

    #[throws]
    pub fn output(&self) -> String {
        let rules = self
            .rules
            .as_ref()
            .ok_or_else(|| anyhow!("No rules applied"))?;
        let mut output = String::new();
        for transaction in &self.transactions {
            let payee = transaction.payee()?;
            let to_account = rules.get(&payee).and_then(|v| v.as_str()).unwrap_or("");
            let flow = transaction.flow()?;
            output.push_str(
                format!(
                    r##"{date} {flag} "{payee}" "{narration}"
  {account} {amount} CNY
  {fund_account}"##,
                    date = transaction.date()?,
                    payee = payee,
                    narration = transaction.narration()?,
                    flag = if to_account.is_empty() || flow.is_unknown() {
                        "!"
                    } else {
                        "*"
                    },
                    account = to_account,
                    amount = transaction.amount()?,
                    fund_account = self.fund_account
                )
                .as_str(),
            );
            output.push('\n');
        }
        output
    }
}
