use clap::{Arg, ArgAction, ArgMatches, Command};

pub fn get_matches() -> ArgMatches {
    Command::new("diane")
        .version("0.1.0") // TODO: Derive this from cargo
        .author("Miguel")
        .about("Note taking app")
        .arg(
            Arg::new("text")
                .value_name("text")
                .help("Input text")
                .required(true)
                .num_args(1),
        )
        .arg(
            Arg::new("omit_newline")
                .short('n')
                .help("Do not print newline")
                .action(ArgAction::SetTrue),
        )
        .get_matches()
}
