mod importers;
mod lib;
#[cfg(test)]
mod test_helpers;

use crate::importers::{alipay, wechat};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Debug)]
enum Source {
    Wechat,
    Alipay,
}

#[derive(Debug, Error)]
#[error("Unknown source type: {0}")]
struct ParseSourceError(String);

impl FromStr for Source {
    type Err = ParseSourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "wechat" => Ok(Source::Wechat),
            "alipay" => Ok(Source::Alipay),
            _ => Err(ParseSourceError(s.to_owned())),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "bean-import", about = "Beancount importer")]
struct Opt {
    /// Activate debug mode
    // short and long flags (-d, --debug) will be deduced from the field's name
    #[structopt(short, long)]
    debug: bool,

    /// Activate edit mode
    // short and long flags (-e, --edit) will be deduced from the field's name
    #[structopt(short, long)]
    edit: bool,

    /// Set source(wechat or alipay)
    #[structopt(short = "s", long = "source", default_value = "wechat")]
    source: Source,
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let bean = match opt.source {
        Source::Alipay => alipay::import(opt.input, opt.edit)?,
        Source::Wechat => wechat::import(opt.input, opt.edit)?,
    };
    match opt.output {
        Some(path) => {
            let mut file = File::create(path)?;
            file.write_all(bean.as_bytes())?;
            println!("Import success!");
        }
        None => println!("{}", bean),
    }
    Ok(())
}
