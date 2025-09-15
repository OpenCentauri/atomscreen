use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "slint-fb",
    about = "Slint with dynamic backend",
)]
pub struct Args {
    #[arg(required = true)]
    pub config: String,
}