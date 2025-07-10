
use clap::Parser;


fn parse_hex_color(hex_str: &str) -> Result<u32, String> {
    let stripped_hex = hex_str.trim_start_matches("0x").trim_start_matches("#");
    if stripped_hex.len() > 6 {
        return Err(format!("Expeced at most 6 hex digits, got {}", stripped_hex.len()));
    }
    let color = u32::from_str_radix(stripped_hex, 16).map_err(|e| e.to_string());
    Ok(color?)
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    #[arg(long, value_name = "FILE", conflicts_with = "color")]
    pub image: Option<String>,

    #[arg(long, value_name = "RGB", value_parser=parse_hex_color, conflicts_with = "image")]
    pub color: Option<u32>
}



