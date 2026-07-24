use diane::app::App;
use diane::config::Config;
use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = match envy::prefixed("DIANE_").from_env::<Config>() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("diane: bad configuration: {e}");
            std::process::exit(1);
        }
    };

    let app = App::new(cfg.diane_root);
    Ok(())
}
