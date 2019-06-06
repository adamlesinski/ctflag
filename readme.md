# Capture the Flag &emsp; [![Latest Version]][crates.io]

[Latest Version]: https://img.shields.io/crates/v/ctflag.svg
[crates.io]: https://crates.io/crates/ctflag

**Capture the Flag is a command-line flag parser Rust library.**

---

## Usage

```rust
use ctflag::{Flags, FromArg, FromArgError, FromArgResult};

#[derive(Flags)]
struct MyFlags {
    #[flag(desc = "The floopy floops the whoop")]
    enable_floopy: bool,

    #[flag(
        desc = "How many slomps to include",
        placeholder = "INTEGER",
        default = 34
    )]
    slomp_count: i64,

    #[flag(desc = "An optional path to a Gmup", placeholder = "PATH")]
    gmup: Option<String>,

    #[flag(desc = "Prints this help message")]
    help: bool,
}

// Custom type.
enum Fruit {
    Apple,
    Orange,
}

impl FromArg for Fruit {
    fn from_arg(s: &str) -> FromArgResult<Self> {
        match s {
            "apple" => Ok(Fruit::Apple),
            "orange" => Ok(Fruit::Orange),
            _ => Err(FromArgError::with_message("must be an apple or orange")),
        }
    }
}

fn main() {
    let result = MyFlags::from_args(std::env::args());
    match result {
        Ok((flags, args)) => {
            if flags.help {
                println!("{}", MyFlags::description());
                return;
            }
            // ...
        }
        Err(err) => {
            println!("Error parsing flags: {}", err);
            println!("{}", MyFlags::description());
        }
    }
}
```

## Setup

```toml
[dependencies]
ctflag = "0.1"
```