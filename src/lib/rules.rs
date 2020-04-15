use super::Transaction;
use anyhow::{anyhow, Context};
use fehler::{throw, throws};
use std::env::var;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::{self, Write};
use std::process;
use toml_edit::{table, value, Document};

static RULES_PATH: &str = "rules.toml";

#[allow(dead_code)]
type Error = anyhow::Error;

pub struct Rules {
    content: Document,
    is_dirty: bool,
}

impl Rules {
    #[throws]
    pub fn from_file() -> Self {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(RULES_PATH)
            .context("Load rules failed")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Self::from_str(&contents)?
    }

    #[throws]
    pub fn from_str(s: &str) -> Self {
        Self {
            content: s.parse().context("Invalid rules.toml contents")?,
            is_dirty: false,
        }
    }

    #[throws]
    fn merge(&mut self, transactions: &[Box<dyn Transaction>]) {
        let root = self.content.as_table_mut();
        let payee_table = root.entry("payee").or_insert(table());
        if let Some(table) = payee_table.as_table_mut() {
            for transaction in transactions {
                let payee = transaction.payee()?;
                if !table.contains_key(&payee) {
                    self.is_dirty = true;
                }
                table.entry(&payee).or_insert(value(""));
            }
        }
    }

    #[throws]
    pub fn merge_with_edit(&mut self, transactions: &[Box<dyn Transaction>]) {
        self.merge(transactions)?;
        if self.is_dirty {
            print!("There are new rules should be specified first, save and edit? (yes/no)");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            match input.trim() {
                "yes" | "y" => {
                    self.save()?;
                    Self::open_editor()?;
                    self.is_dirty = false;
                }
                _ => throw!(anyhow!("Exit")),
            }
        }
    }

    #[throws]
    fn save(&self) {
        let mut rules_file = OpenOptions::new().append(true).open(RULES_PATH)?;
        let content = self.content.to_string_in_original_order();
        rules_file.write_all(content.as_bytes())?;
    }

    #[throws]
    fn open_editor() {
        let editor = var("EDITOR").context("Unable to read $EDITOR")?;
        process::Command::new(editor).arg(RULES_PATH).status()?;
    }

    pub fn get_payee_account(&self, payee: &str) -> Option<&str> {
        self.content["payee"]
            .as_table()
            .and_then(|table| table[payee].as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::Rules;
    use crate::test_helpers::MockTransanction;
    use anyhow;
    use fehler::throws;

    type Error = anyhow::Error;

    #[throws]
    #[test]
    fn test_from_file() {
        let rules = Rules::from_file()?;
        assert_eq!(rules.is_dirty, false);
    }

    #[throws]
    #[test]
    fn test_merge_to_empty() {
        let mut rules = Rules::from_str("")?;
        let transactions = vec![MockTransanction::gen_with_payee("test")];
        rules.merge(&transactions)?;
        assert_eq!(rules.is_dirty, true);
        assert_eq!(
            rules.content.to_string(),
            r#"
[payee]
test = ""
"#
        );
    }

    #[throws]
    #[test]
    fn test_merge_to_exist() {
        let mut rules = Rules::from_str(
            r#"
[payee]
test = "existed"
"#,
        )?;
        let transactions = vec![
            MockTransanction::gen_with_payee("test"),
            MockTransanction::gen_with_payee("newone"),
        ];
        rules.merge(&transactions)?;
        assert_eq!(
            rules.content.to_string(),
            r#"
[payee]
test = "existed"
newone = ""
"#
        );
    }

    #[throws]
    #[test]
    fn test_get_payee_account() {
        let rules = Rules::from_str(
            r#"
[payee]
test = "Expense:Test"
"#,
        )?;
        assert_eq!(rules.get_payee_account("test"), Some("Expense:Test"));
    }
}
