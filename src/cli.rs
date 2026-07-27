use clap::Parser;

#[derive(Parser)]
#[command(name = "diane")]
#[command(about = "Note taking app")]
pub struct Cli {
    pub input_text: Option<String>,
}
