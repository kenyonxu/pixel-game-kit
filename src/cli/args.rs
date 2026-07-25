use crate::palette::parse_palette_hex;
use crate::{Config, PixelSnapperError, Result};
use std::env;

#[derive(Debug)]
pub enum CliCommand {
    Run(Config),
    Help,
    Version,
}

/// Internal entry point used by the packaged CLI binary.
#[doc(hidden)]
pub fn run_cli() -> std::process::ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    match parse_cli_args(&args) {
        Ok(CliCommand::Help) => {
            print_cli_help();
            std::process::ExitCode::SUCCESS
        }
        Ok(CliCommand::Version) => {
            println!("pixel-game-kit {}", env!("CARGO_PKG_VERSION"));
            std::process::ExitCode::SUCCESS
        }
        Ok(CliCommand::Run(config)) => match crate::cli::process(&config) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Error: {error}");
                std::process::ExitCode::from(1)
            }
        },
        Err(error) => {
            eprintln!("Error: {error}");
            eprintln!("Run 'pixel-game-kit --help' for usage.");
            std::process::ExitCode::from(2)
        }
    }
}

pub fn print_cli_help() {
    println!(
        concat!(
            "Pixel Game Kit {version}\n",
            "Fix inconsistent pixel art by detecting and snapping it to its implicit grid.\n\n",
            "USAGE:\n",
            "  pixel-game-kit <INPUT> <OUTPUT> [COLORS] [OPTIONS]\n\n",
            "ARGUMENTS:\n",
            "  <INPUT>   Input PNG/JPEG file, or a directory for batch processing\n",
            "  <OUTPUT>  Output PNG file, or a different output directory for a batch\n",
            "  [COLORS]  Number of palette colors [default: 16]\n\n",
            "OPTIONS:\n",
            "  --pixel-size <PIXELS>                       Override the auto-detected pixel size\n",
            "  --palette <HEX,...>                         Use comma-separated 6-digit hex palette colors\n",
            "  --detect <auto|runs|tiled|elastic>          Grid detection strategy [default: auto]\n",
            "  --resample <majority|median|dominant|mode|qvote>  Grid-cell reduction [default: majority]\n",
            "  --sample-window <1-9>                       Median neighborhood [default: 3]\n",
            "  --colorspace <rgb|oklab>                    Quantize colorspace [default: oklab]\n",
            "  --dither <none|fs|bayer2|bayer4|bayer8|ordered>  Dithering [default: none]\n",
            "  --dither-strength <0-2>                     Dither strength [default: 1.0]\n",
            "  --preset <name>                             Snap to preset palette [default: none]\n",
            "  --bg-remove                                Enable background removal\n",
            "  --bg-tolerance <0-255>                     Per-channel bg tolerance [default: 64]\n",
            "  --bg-connectivity <4|8>                    Flood connectivity [default: 4]\n",
            "  --bg-scope <outer|all>                     Removal scope [default: outer]\n",
            "  --bg-floating-threshold <n>                Floating-island cleanup size (0=off) [default: 0]\n",
            "  --outline <rounded|sharp>                  Outline style [default: off]\n",
            "  --outline-color <hex>                      Outline color [default: 000000]\n",
            "  --morph                                    Enable 2x2 open->close (alpha-only)\n",
            "  --alpha-threshold <n|auto>                 Alpha binarize (strict >) [default: off]\n",
            "  --json                                      Output detection candidates as JSON instead of processing\n",
            "  -h, --help                                  Print help\n",
            "  -V, --version                               Print version\n\n",
            "EXAMPLES:\n",
            "  pixel-game-kit input.png output.png\n",
            "  pixel-game-kit input.png output.png 16 --pixel-size 8\n",
            "  pixel-game-kit inputs outputs --palette 0d2b45,ffecd6"
        ),
        version = env!("CARGO_PKG_VERSION")
    );
}

pub fn parse_cli_args(args: &[String]) -> Result<CliCommand> {
    if args.is_empty()
        || args
            .iter()
            .any(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        return Ok(CliCommand::Help);
    }
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-V" | "--version"))
    {
        return Ok(CliCommand::Version);
    }
    if args.len() < 2 {
        return Err(PixelSnapperError::InvalidInput(
            "missing output path".to_string(),
        ));
    }

    let mut config = Config {
        input_path: args[0].clone(),
        output_path: args[1].clone(),
        ..Default::default()
    };

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--pixel-size" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--pixel-size requires a value".to_string(),
                    ));
                };

                match val.parse::<f64>() {
                    Ok(px) if px.is_finite() && px > 0.0 => config.pixel_size_override = Some(px),
                    _ => {
                        return Err(PixelSnapperError::InvalidInput(format!(
                            "invalid --pixel-size '{}': expected a positive number",
                            val
                        )))
                    }
                }
                i += 2;
            }
            "--palette" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--palette requires a value".to_string(),
                    ));
                };

                config.palette = Some(parse_palette_hex(val)?);
                i += 2;
            }
            "--detect" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--detect requires a value".to_string(),
                    ));
                };
                config.detect_strategy = match val.as_str() {
                    "auto" => crate::detect::DetectStrategy::Auto,
                    "runs" => crate::detect::DetectStrategy::Runs,
                    "tiled" => crate::detect::DetectStrategy::Tiled,
                    "elastic" => crate::detect::DetectStrategy::Elastic,
                    _ => {
                        return Err(PixelSnapperError::InvalidInput(format!(
                            "invalid --detect '{}' (expected auto|runs|tiled|elastic)",
                            val
                        )))
                    }
                };
                i += 2;
            }
            "--resample" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--resample requires a value".to_string(),
                    ));
                };
                config.resample_method = match val.as_str() {
                    "majority" => crate::resample::ResampleMethod::Majority,
                    "median" => crate::resample::ResampleMethod::Median,
                    "dominant" => crate::resample::ResampleMethod::Dominant,
                    "mode" => crate::resample::ResampleMethod::Mode,
                    "qvote" => crate::resample::ResampleMethod::Qvote,
                    _ => return Err(PixelSnapperError::InvalidInput(format!(
                        "invalid --resample '{}' (expected majority|median|dominant|mode|qvote)",
                        val
                    ))),
                };
                i += 2;
            }
            "--sample-window" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--sample-window requires a value".to_string(),
                    ));
                };
                match val.parse::<usize>() {
                    Ok(n) if (1..=9).contains(&n) => config.resample_sample_window = n,
                    _ => {
                        return Err(PixelSnapperError::InvalidInput(format!(
                            "invalid --sample-window '{}' (expected 1-9)",
                            val
                        )))
                    }
                }
                i += 2;
            }
            "--json" => {
                config.json_output = true;
                i += 1;
            }
            "--colorspace" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--colorspace requires a value".to_string(),
                    ));
                };
                config.quantize_colorspace = match val.as_str() {
                    "rgb" => crate::quantize::Colorspace::Rgb,
                    "oklab" => crate::quantize::Colorspace::Oklab,
                    _ => {
                        return Err(PixelSnapperError::InvalidInput(format!(
                            "invalid --colorspace '{}' (expected rgb|oklab)",
                            val
                        )))
                    }
                };
                i += 2;
            }
            "--dither" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--dither requires a value".to_string(),
                    ));
                };
                config.quantize_dither = match val.as_str() {
                    "none" => crate::quantize::DitherMethod::None,
                    "fs" => crate::quantize::DitherMethod::FloydSteinberg,
                    "bayer2" => crate::quantize::DitherMethod::Bayer2,
                    "bayer4" => crate::quantize::DitherMethod::Bayer4,
                    "bayer8" => crate::quantize::DitherMethod::Bayer8,
                    "ordered" => crate::quantize::DitherMethod::Ordered,
                    _ => {
                        return Err(PixelSnapperError::InvalidInput(format!(
                            "invalid --dither '{}' (expected none|fs|bayer2|bayer4|bayer8|ordered)",
                            val
                        )))
                    }
                };
                i += 2;
            }
            "--dither-strength" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--dither-strength requires a value".to_string(),
                    ));
                };
                match val.parse::<f64>() {
                    Ok(s) if (0.0..=2.0).contains(&s) => config.quantize_dither_strength = s,
                    _ => {
                        return Err(PixelSnapperError::InvalidInput(format!(
                            "invalid --dither-strength '{}' (expected 0-2)",
                            val
                        )))
                    }
                }
                i += 2;
            }
            "--preset" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--preset requires a value".to_string(),
                    ));
                };
                config.quantize_preset_palette = match val.as_str() {
                    "none" => crate::quantize::PresetPalette::None,
                    "nes" => crate::quantize::PresetPalette::Nes,
                    "gameboy" => crate::quantize::PresetPalette::GameBoy,
                    "sgb" => crate::quantize::PresetPalette::Sgb,
                    "snes" => crate::quantize::PresetPalette::Snes,
                    "pc9801" => crate::quantize::PresetPalette::Pc9801,
                    "msx1" => crate::quantize::PresetPalette::Msx1,
                    "pico8" => crate::quantize::PresetPalette::Pico8,
                    "sweetie16" => crate::quantize::PresetPalette::Sweetie16,
                    "endesga32" => crate::quantize::PresetPalette::Endesga32,
                    _ => {
                        return Err(PixelSnapperError::InvalidInput(format!(
                            "invalid --preset '{}'",
                            val
                        )))
                    }
                };
                i += 2;
            }
            "--bg-remove" | "--remove-bg" => {
                config.post_bg_remove = true;
                i += 1;
            }
            "--bg-tolerance" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--bg-tolerance requires a value".to_string(),
                    ));
                };
                match val.parse::<u8>() {
                    Ok(n) => config.post_bg_tolerance = n,
                    _ => {
                        return Err(PixelSnapperError::InvalidInput(format!(
                            "invalid --bg-tolerance '{}' (expected 0-255)",
                            val
                        )))
                    }
                }
                i += 2;
            }
            "--bg-connectivity" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--bg-connectivity requires a value".to_string(),
                    ));
                };
                config.post_bg_connectivity = match val.as_str() {
                    "4" => crate::postprocess::BgConnectivity::Conn4,
                    "8" => crate::postprocess::BgConnectivity::Conn8,
                    _ => {
                        return Err(PixelSnapperError::InvalidInput(format!(
                            "invalid --bg-connectivity '{}' (expected 4|8)",
                            val
                        )))
                    }
                };
                i += 2;
            }
            "--bg-scope" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--bg-scope requires a value".to_string(),
                    ));
                };
                config.post_bg_scope = match val.as_str() {
                    "outer" => crate::postprocess::BgScope::Outer,
                    "all" => crate::postprocess::BgScope::All,
                    _ => {
                        return Err(PixelSnapperError::InvalidInput(format!(
                            "invalid --bg-scope '{}' (expected outer|all)",
                            val
                        )))
                    }
                };
                i += 2;
            }
            "--bg-floating-threshold" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--bg-floating-threshold requires a value".to_string(),
                    ));
                };
                match val.parse::<usize>() {
                    Ok(n) => config.post_bg_floating_max_pixels = n,
                    _ => {
                        return Err(PixelSnapperError::InvalidInput(format!(
                            "invalid --bg-floating-threshold '{}' (expected non-negative integer)",
                            val
                        )))
                    }
                }
                i += 2;
            }
            "--outline" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--outline requires a value".to_string(),
                    ));
                };
                config.post_outline = match val.as_str() {
                    "rounded" => crate::postprocess::OutlineStyle::Rounded,
                    "sharp" => crate::postprocess::OutlineStyle::Sharp,
                    _ => {
                        return Err(PixelSnapperError::InvalidInput(format!(
                            "invalid --outline '{}' (expected rounded|sharp)",
                            val
                        )))
                    }
                };
                i += 2;
            }
            "--outline-color" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--outline-color requires a value".to_string(),
                    ));
                };
                config.post_outline_color = parse_hex_color(val)?;
                i += 2;
            }
            "--morph" | "--morphology" => {
                config.post_morph = true;
                i += 1;
            }
            "--alpha-threshold" => {
                let Some(val) = args.get(i + 1) else {
                    return Err(PixelSnapperError::InvalidInput(
                        "--alpha-threshold requires a value".to_string(),
                    ));
                };
                config.post_alpha_threshold = match val.as_str() {
                    "auto" => crate::postprocess::AlphaThreshold::Auto,
                    n => match n.parse::<u8>() {
                        Ok(t) => crate::postprocess::AlphaThreshold::Fixed(t),
                        _ => {
                            return Err(PixelSnapperError::InvalidInput(format!(
                                "invalid --alpha-threshold '{}' (expected 0-255 or auto)",
                                val
                            )))
                        }
                    },
                };
                i += 2;
            }
            "--binarize" => {
                config.post_alpha_threshold = crate::postprocess::AlphaThreshold::Fixed(128);
                i += 1;
            }
            arg if arg.starts_with("--") => {
                return Err(PixelSnapperError::InvalidInput(format!(
                    "unknown argument '{}'",
                    arg
                )));
            }
            k_arg => {
                match k_arg.parse::<usize>() {
                    Ok(k) if k > 0 => config.k_colors = k,
                    _ => {
                        return Err(PixelSnapperError::InvalidInput(format!(
                            "invalid color count '{}': expected a positive integer",
                            k_arg
                        )))
                    }
                }
                i += 1;
            }
        }
    }

    Ok(CliCommand::Run(config))
}

fn parse_hex_color(s: &str) -> Result<[u8; 3]> {
    let hex = s.trim_start_matches('#');
    if hex.len() != 6 {
        return Err(PixelSnapperError::InvalidInput(format!(
            "invalid outline color '{}' (expected 6-digit hex, e.g. ff00ff)",
            s
        )));
    }
    let parse_channel = |range: std::ops::Range<usize>| -> Result<u8> {
        u8::from_str_radix(&hex[range.start..range.end], 16).map_err(|_| {
            PixelSnapperError::InvalidInput(format!(
                "invalid outline color '{}' (expected 6-digit hex)",
                s
            ))
        })
    };
    Ok([
        parse_channel(0..2)?,
        parse_channel(2..4)?,
        parse_channel(4..6)?,
    ])
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_color_accepts_hash_and_plain() {
        assert_eq!(parse_hex_color("#ff00aa").unwrap(), [255, 0, 170]);
        assert_eq!(parse_hex_color("00aaFF").unwrap(), [0, 170, 255]);
    }

    #[test]
    fn parse_hex_color_rejects_invalid() {
        assert!(parse_hex_color("gg00aa").is_err());
        assert!(parse_hex_color("ff00a").is_err());
    }
}
