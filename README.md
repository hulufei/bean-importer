# bean-importer

Currently support:

- Wechat
- Alipay

## Usage

```
USAGE:
    bean-importer [FLAGS] [OPTIONS] <input> [output]

FLAGS:
    -d, --debug      Activate debug mode
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s, --source <source>    Set source(wechat or alipay) [default: wechat]

ARGS:
    <input>     Input file
    <output>    Output file, stdout if not present
```

## rules.toml

The importer will generate a `rules.toml` to let you specify convert rules for your transactions.

Currently, the rule is stupid simple, just key value pairs. The key is `payee`, value is `account`.
