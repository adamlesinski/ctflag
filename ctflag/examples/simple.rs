use ctflag::Flags;
use std::env;

#[derive(Debug, Flags)]
struct MyFlags {
    #[flag(desc = "Whether this is active", default = true)]
    is_active: bool,

    #[flag(placeholder = "ADDRESS")]
    opt_address: Option<String>,

    #[flag(short = 'a', default = "1.2.3.4")]
    address: String,

    #[flag(placeholder = "INTEGER", desc = "A good number.", default = 34)]
    number: i64,

    #[flag(placeholder = "SIZE")]
    size: Option<bool>,

    #[flag(default = "one")]
    thing: Thing,

    #[flag(desc = "Prints this help message")]
    help: bool,
}

#[derive(Debug)]
enum Thing {
    One,
    Two,
}

impl ctflag::FromArg for Thing {
    fn from_arg(s: &str) -> ctflag::FromArgResult<Thing> {
        match s {
            "one" => Ok(Thing::One),
            "two" => Ok(Thing::Two),
            _ => Err(ctflag::FromArgError::with_message("must be one or two")),
        }
    }
}

fn main() {
    let result = MyFlags::from_args(env::args());
    match result {
        Ok((flags, args)) => {
            println!("{:?}", flags);
            println!("{:?}", args);
            if flags.help {
                println!("{}", MyFlags::description());
            }
        }
        Err(e) => {
            println!("Error parsing flags: {}", e);
            println!("{}", MyFlags::description());
        }
    }
}
