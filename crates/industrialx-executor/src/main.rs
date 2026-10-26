use std::process::ExitCode;

fn main() -> ExitCode {
    let mut port: u16 = 8787;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match (arg.as_str(), args.next()) {
            ("--port", Some(value)) => match value.parse() {
                Ok(parsed) => port = parsed,
                Err(_) => {
                    eprintln!("--port needs a number");
                    return ExitCode::from(2);
                }
            },
            _ => {
                eprintln!("usage: industrialx-executor [--port <n>]  (binds 127.0.0.1 only)");
                return ExitCode::from(2);
            }
        }
    }
    match industrialx_executor::http::serve(port) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("industrialx-executor: {error}");
            ExitCode::from(1)
        }
    }
}
