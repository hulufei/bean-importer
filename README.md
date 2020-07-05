# bean-importer

Currently support:

- Wechat
- [Alipay](https://consumeprod.alipay.com/record/advanced.htm)

## Usage

```
USAGE:
    bean-importer [FLAGS] [OPTIONS] <input> [output]

FLAGS:
    -d, --debug      Activate debug mode
    -e, --edit       Activate edit mode
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s, --source <source>    Set source(wechat or alipay) [default: wechat]

ARGS:
    <input>     Input file
    <output>    Output file, stdout if not present
```

## rules.toml

The importer will generate a `rules.toml` to let you specify transform rules for your transactions.

The rules like:

```toml
[fund]
"零钱" = "Assets:Wechat"
"Some Bank Name" = "Assets:Bank:SomeBank"

[payee]
"Some Restaurant" = 'Expenses:Account'
"Another name for some payee" = { alias = 'unified', account = 'Expenses:Unified' }
```

The `fund` section specify fund source accounts, generally it will be `Assets` accounts. The `payee` section
specify payee accounts, generally it will be `Expenses` accounts.

The `payee` section support additional `alias` rule to change the payee value, it's good for unifing some payees
have multiple accounts produce different transactions.(eg. have both wechat and alipay accounts)
