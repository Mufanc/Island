#[macro_export]
macro_rules! check_err {
    ( $name:ident$($body:tt)* ) => {
        match $name$($body)* {
            Err(err) => {
                eprintln!("{}: {}", stringify!($name), err.desc());
                std::process::exit(1)
            }
            Ok(result) => result
        }
    };
}
