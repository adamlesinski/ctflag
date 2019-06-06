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
