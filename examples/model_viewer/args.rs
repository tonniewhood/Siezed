use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    #[arg(long, value_name = "FILE")]
    pub model: String,

    #[arg(short, value_name = "s")]
    pub scale_factor: f32,
}
