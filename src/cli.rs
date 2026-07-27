use clap::{ArgMatches, Command};

pub fn GetMatches() -> ArgMatches {
    Command::new("diane")
        .version("0.1.0") // TODO: Derive this from cargo
        .author("Miguel")
        .about("Note taking app")
        .get_matches()
}
