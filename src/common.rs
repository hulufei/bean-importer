use csv::StringRecord;
use std::collections::HashMap;
use std::convert::From;
use std::env::var;
use std::error::Error;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::process;
use toml::Value;

static RULES_PATH: &str = "rules.toml";

pub struct Alipay {
    record: StringRecord,
}

impl Alipay {
    pub fn new(record: StringRecord) -> Self {
        Self { record }
    }
}

pub struct Wechat {
    record: StringRecord,
}

impl Wechat {
    pub fn new(record: StringRecord) -> Self {
        Self { record }
    }
}

pub enum TrasactionKind {
    Income,
    Expense,
    Unknown,
}

impl From<Option<&str>> for TrasactionKind {
    fn from(s: Option<&str>) -> Self {
        match s {
            Some("收入") => TrasactionKind::Income,
            Some("支出") => TrasactionKind::Expense,
            _ => TrasactionKind::Unknown,
        }
    }
}

pub struct Trasaction {
    date: String,
    payee: String,
    narration: String,
    amount: f32,
    kind: TrasactionKind,
}

impl Trasaction {
    fn get_amount(&self) -> f32 {
        match self.kind {
            TrasactionKind::Income => -self.amount,
            _ => self.amount,
        }
    }
    fn is_unknown(&self) -> bool {
        match self.kind {
            TrasactionKind::Unknown => true,
            _ => false,
        }
    }
}

impl From<Alipay> for Trasaction {
    fn from(alipay: Alipay) -> Self {
        let Alipay { record } = alipay;
        let date = record.get(3).unwrap();
        Self {
            date: date.split_whitespace().next().unwrap().to_owned(),
            payee: record.get(7).unwrap().to_owned(),
            narration: record.get(8).unwrap().to_owned(),
            amount: record.get(9).unwrap().parse().unwrap(),
            kind: TrasactionKind::from(record.get(10)),
        }
    }
}

impl From<Wechat> for Trasaction {
    fn from(wechat: Wechat) -> Self {
        let Wechat { record } = wechat;
        let date = record.get(0).unwrap();
        let amount = record.get(5).unwrap().trim_start_matches('¥');
        Self {
            date: date.split_whitespace().next().unwrap().to_owned(),
            payee: record.get(2).unwrap().to_owned(),
            narration: record.get(3).unwrap().to_owned(),
            amount: amount.parse().unwrap(),
            kind: TrasactionKind::from(record.get(4)),
        }
    }
}

pub struct Bean {
    transactions: Vec<Trasaction>,
    base_account: String,
}

impl Bean {
    pub fn new(account: &str) -> Self {
        Self {
            transactions: Vec::new(),
            base_account: account.to_owned(),
        }
    }

    pub fn add(&mut self, transaction: Trasaction) {
        self.transactions.push(transaction);
    }
}

impl fmt::Display for Bean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match load_rules() {
            Ok((_, rules)) => {
                for transaction in &self.transactions {
                    let account = rules.get(&transaction.payee).unwrap().as_str().unwrap();
                    writeln!(
                        f,
                        r##"{date} {flag} "{payee}" "{narration}"
  {account} {amount} CNY
  {base_account}"##,
                        date = transaction.date,
                        payee = transaction.payee,
                        narration = transaction.narration,
                        flag = if account.is_empty() || transaction.is_unknown() {
                            "!"
                        } else {
                            "*"
                        },
                        account = account,
                        amount = transaction.get_amount(),
                        base_account = self.base_account
                    )?;
                }
                Ok(())
            }
            Err(e) => panic!("Load rules failed {:?}", e),
        }
    }
}

pub fn load_rules() -> Result<(File, Value), Box<dyn Error>> {
    let mut file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(RULES_PATH)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let rules = contents.parse::<Value>()?;
    Ok((file, rules))
}

pub fn handle_new_rules(
    mut rules_file: File,
    new_rules: HashMap<String, &str>,
) -> Result<(), Box<dyn Error>> {
    let has_empty = new_rules.values().any(|v| v.is_empty());
    if has_empty {
        print!("There are new rules should be specified first, save and edit? (yes/no)");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            match input.trim() {
                "yes" | "y" => {
                    // Append new rules
                    let contents = new_rules
                        .iter()
                        .map(|(k, v)| format!("'{}' = '{}'", k, v))
                        .collect::<Vec<_>>()
                        .join("\n");
                    rules_file.write_all(b"\n")?;
                    rules_file.write_all(contents.as_bytes())?;
                    // edit
                    let editor = var("EDITOR").unwrap();
                    let status = process::Command::new(editor)
                        .arg(RULES_PATH)
                        .status()
                        .expect("Open $EDITOR failed");
                    if status.success() {
                        return Ok(());
                    }
                    panic!("Edit rules failed");
                }
                _ => process::exit(0),
            }
        }
    }
    Ok(())
}
