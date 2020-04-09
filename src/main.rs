mod common;
mod importer_csv;

use importer_csv::{import_alipay, import_wechat};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug)]
enum Source {
    Wechat,
    Alipay,
}

#[derive(Debug)]
struct ParseSourceError(String);

impl fmt::Display for ParseSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown source {}", self.0)
    }
}

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

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let bean = match opt.source {
        Source::Alipay => import_alipay(opt.input)?,
        Source::Wechat => import_wechat(opt.input)?,
    };
    match opt.output {
        Some(path) => {
            let mut file = File::create(path)?;
            file.write_all(bean.to_string().as_bytes())?;
            println!("Import success!");
        }
        None => println!("{}", bean),
    }
    Ok(())
}
