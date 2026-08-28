//! Deterministic M0 command-line boundary; transport behavior is intentionally absent.

const USAGE: &str = "Usage: neko <client|server|probe> [--help]\n\nM0 boundary scaffold; no transport is implemented.\n";

fn print_help(command: Option<&str>) {
    match command {
        Some(name) => println!("neko {name}: M0 boundary scaffold; no transport is implemented\n\nUsage: neko {name} [--help]"),
        None => print!("{USAGE}"),
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let command = args.next();
    match command.as_deref() {
        None => print_help(None),
        Some("client" | "server" | "probe") => {
            let command_name = command.as_deref().expect("matched command");
            match args.next().as_deref() {
                None | Some("--help") => print_help(Some(command_name)),
                Some(flag) => {
                    eprintln!("neko {command_name}: unexpected argument `{flag}`");
                    eprintln!("Usage: neko {command_name} [--help]");
                    std::process::exit(2);
                }
            }
        }
        Some("--help") => print_help(None),
        Some(command_name) => {
            eprintln!("neko: unknown command `{command_name}`");
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_mentions_all_scaffold_commands() {
        assert!(USAGE.contains("client"));
        assert!(USAGE.contains("server"));
        assert!(USAGE.contains("probe"));
    }
}
