## Pumpman

Open source pump.fun services in rust.


1. edit the config file `config.example.toml`
2. rename `config.example.toml` to `config.toml`
3. `cargo b --release`
4. `./target/release/pumpman --config config.toml`

## Usage

``` sh
Replika command line interfaces

Usage: replika [OPTIONS] <COMMAND>

Commands:
  start           Start all services
  sig             Prints transaction from signature
  coin            Prints metadata of a token
  dex             Prints pairs of a token
  info            Get alert info of a token
  token-accounts  Get details of token account
  bonding-curve   Get bonding curve of pumpfun coin
  verify          Verify signature
  sign            Sign message
  sim-bump        Simulate bump
  sim-withdraw    Simulate withdraw
  pump-fee        Get pumpfun fee
  balance         Get balance
  import          Import wallet to database
  comment         Run the commenter
  init            Init database
  help            Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>  Path of replika config [default: config.toml]
  -u, --update           If update cache
  -v, --verbose...       The verbosity level
  -h, --help             Print help
```



## Scripts

``` sh
docker build -t pumpman . --platform=linux/amd64
```

``` sh
diesel --database-url postgres://localhost/takeover print-schema > src/schema.rs
```
