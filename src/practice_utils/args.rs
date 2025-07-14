use clap::Parser;

fn parse_hex_color(hex_str: &str) -> Result<u32, String> {
    let stripped_hex = hex_str.trim_start_matches("0x").trim_start_matches("#");
    if stripped_hex.len() > 6 {
        return Err(format!(
            "Expeced at most 6 hex digits, got {}",
            stripped_hex.len()
        ));
    }
    let color = u32::from_str_radix(stripped_hex, 16).map_err(|e| e.to_string());
    Ok(color?)
}

#[derive(Clone, Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    #[arg(long, value_name = "FILE", default_value = "")]
    pub image: Option<String>,

    #[arg(long, value_name = "RGB", value_parser=parse_hex_color, default_value="0")]
    pub color: Option<u32>,

    #[arg(long, help = "Unlock the aspect ratio")]
    pub no_aspect: bool,
}
