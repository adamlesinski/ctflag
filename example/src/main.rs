use rarg::Flags;
use std::env;

#[derive(Flags)]
struct MyFlags {
    is_active: bool,
    address: Option<String>,
}

fn parse_flags(args: env::Args) -> Result<MyFlags, rarg::ParseError> {
    let mut arg_iter = args.into_iter();
    // Skip the first arg which is the program name.
    arg_iter.next();
    let mut parsed_is_active: bool = Default::default();
    let mut parsed_address: Option<String> = Default::default();
    while let Some(arg) = arg_iter.next() {
        let mut arg_name = arg.as_ref();
        let mut arg_value = None;
        if let Some(idx) = arg.find('=') {
            arg_name = &arg[0..idx];
            arg_value = Some(&arg[idx + 1..arg.len()]);
        }
        match arg_name.as_ref() {
            "--is_active" => {
                let mut result = true;
                if let Some(value) = arg_value {
                    result = value.parse::<bool>().map_err(|_| {
                        rarg::ParseError::FlagError(format!(
                            "failed to parse '{}' as type bool",
                            value
                        ))
                    })?;
                }
                parsed_is_active = result;
            }
            "--address" => match arg_value {
                Some(value) => {
                    parsed_address = Some(value.to_string());
                }
                None => {
                    return Err(rarg::ParseError::FlagError(format!(
                        "No value for '{}'",
                        arg
                    )))
                }
            },
            _ => {
                panic!("for now");
            }
        }
    }
    Ok(MyFlags {
        is_active: parsed_is_active,
        address: parsed_address,
    })
}

fn main() {
    let result = MyFlags::from_args(env::args());
    match result {
        Ok(_flags) => println!("ok"),
        Err(e) => {
            println!("Error parsing flags: {:?}", e);
            println!("{}", MyFlags::description());
        }
    }

    let result = parse_flags(env::args());
    match result {
        Ok(flags) => {
            println!("is_active={:?}", flags.is_active);
            println!("address={:?}", flags.address);
        }
        Err(e) => {
            println!("Error parsing flags: {:?}", e);
            println!("{}", MyFlags::description());
        }
    }
}
