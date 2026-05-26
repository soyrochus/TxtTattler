use std::path::PathBuf;

use clap::{ArgAction, Parser};

use crate::domain::entities::{SpeechModelName, VoiceName};

const HELP_HEADER: &str = r#"
📢 TxtTattler
  _____        _  _____     _   _   _           
 |_   _|_  ___| ||_   _|_ _| |_| |_| | ___ _ __ 
   | | \ \/ / | __|| |/ _` | __| __| |/ _ \ '__|
   | |  >  <| | |_ | | (_| | |_| |_| |  __/ |   
   |_| /_/\_\_|\__||_|\__,_|\__|\__|_|\___|_|   

Your gossipy little file reader that loves to narrate.
"#;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "txttattler",
    version,
    about = "Reads a text file aloud with OpenAI or Azure OpenAI TTS.",
    before_help = HELP_HEADER,
    after_help = "Example: txttattler story.txt --voice nova --model tts-1-hd --output story.mp3",
    color = clap::ColorChoice::Always,
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(value_name = "FILE", required_unless_present = "list_voices")]
    pub file: Option<PathBuf>,

    #[arg(short = 'v', long, value_name = "VOICE", value_parser = clap::builder::PossibleValuesParser::new(VoiceName::ALL.map(VoiceName::as_str)))]
    pub voice: Option<String>,

    #[arg(short = 'm', long, value_name = "MODEL", value_parser = clap::builder::PossibleValuesParser::new(SpeechModelName::ALL.map(SpeechModelName::as_str)))]
    pub model: Option<String>,

    #[arg(short = 's', long, value_name = "FLOAT", default_value_t = 1.0, value_parser = parse_speed)]
    pub speed: f32,

    #[arg(short = 'o', long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    #[arg(long, action = ArgAction::SetTrue)]
    pub no_play: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub no_cache: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub refresh: bool,

    #[arg(long, value_name = "PATH")]
    pub cache_dir: Option<PathBuf>,

    #[arg(long, action = ArgAction::SetTrue)]
    pub azure: bool,

    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[arg(long, action = ArgAction::SetTrue)]
    pub list_voices: bool,

    #[arg(short = 'V', long, action = ArgAction::SetTrue)]
    pub verbose: bool,
}

pub fn print_available_voices() {
    println!("Available voices:");
    for voice in VoiceName::ALL {
        println!(" - {}", voice.as_str());
    }
}

fn parse_speed(value: &str) -> Result<f32, String> {
    let speed: f32 = value
        .parse()
        .map_err(|_| "Speed must be a floating-point number.".to_string())?;
    if !(0.25..=4.0).contains(&speed) {
        return Err("Speed must be between 0.25 and 4.0.".to_string());
    }
    Ok(speed)
}
