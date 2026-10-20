use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    #[arg(long, value_name = "FILE")]
    pub mesh: String,

    #[arg(
        long,
        default_value = "false",
        help = "Whether triangles will be wire frames or not"
    )]
    pub wire: bool,
}
