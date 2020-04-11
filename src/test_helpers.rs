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
