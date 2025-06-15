use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use std::{error::Error, io, thread, time::Duration};
use clap::Parser;
use ctrlc;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// GPIO chip path (e.g., /dev/gpiochip0)
    #[arg(long, default_value = "/dev/gpiochip0")]
    chip: String,

    /// GPIO line offset (e.g., 50)
    #[arg(long)]
    offset: u32,

    /// Temperature (°C) to turn the fan ON
    #[arg(long, default_value_t = 60.0)]
    on_temp: f32,

    /// Temperature (°C) to turn the fan OFF
    #[arg(long, default_value_t = 45.0)]
    off_temp: f32,

    /// Interval in milliseconds
    #[arg(long, default_value_t = 2000)]
    interval: u64,

    /// Enable verbose logging
    #[arg(long)]
    verbose: bool,
}

fn read_temp_millicelsius() -> io::Result<i32> {
    let contents = std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")?;
    contents.trim()
        .parse::<i32>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn request_output_line(chip_path: &str, offset: u32) -> io::Result<(Chip, LineHandle)> {
    let mut chip = Chip::new(chip_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Chip error: {}", e)))?;

    let handle = chip
        .get_line(offset)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Line error: {}", e)))?
        .request(LineRequestFlags::OUTPUT, 0, "fan-control-rs")
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Request error: {}", e)))?;

    Ok((chip, handle))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let running = true;

    // Handle Ctrl+C
    ctrlc::set_handler(move || {
        println!("Shutting down gracefully...");
        std::process::exit(0);
    })?;

    let (_chip, handle) = request_output_line(
        &args.chip,
        args.offset)?;

    let on_temp_mc = (args.on_temp * 1000.0) as i32;
    let off_temp_mc = (args.off_temp * 1000.0) as i32;
    let interval = Duration::from_millis(args.interval);
    let mut fan_on = false;

    while running {
        let temp = read_temp_millicelsius()?;

        if args.verbose {
            println!("Temp: {:.1}°C | Fan: {}",
                     temp as f32 / 1000.0,
                     if fan_on { "ON" } else { "OFF" });
        }

        if fan_on && temp <= off_temp_mc {
            handle.set_value(0)?;
            fan_on = false;
        } else if !fan_on && temp >= on_temp_mc {
            handle.set_value(1)?;
            fan_on = true;
        }

        thread::sleep(interval);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_thresholds() {
        let on_temp_c = 60.0;
        let off_temp_c = 45.0;
        let on_temp_mc = (on_temp_c * 1000.0) as i32;
        let off_temp_mc = (off_temp_c * 1000.0) as i32;

        assert_eq!(on_temp_mc, 60000);
        assert_eq!(off_temp_mc, 45000);
    }

    #[test]
    fn test_cli_parsing_defaults() {
        let args = Args::parse_from(["test", "--offset", "0"]);
        assert_eq!(args.chip, "/dev/gpiochip0");
        assert_eq!(args.offset, 0);
        assert_eq!(args.on_temp, 60.0);
        assert_eq!(args.off_temp, 45.0);
        assert_eq!(args.interval, 2000);
    }

    #[test]
    fn test_cli_parsing_custom_values() {
        let args = Args::parse_from([
            "test",
            "--chip",
            "/dev/gpiochip1",
            "--offset",
            "50",
            "--on-temp",
            "70",
            "--off-temp",
            "50",
            "--interval",
            "1000",
        ]);

        assert_eq!(args.chip, "/dev/gpiochip1");
        assert_eq!(args.offset, 50);
        assert_eq!(args.on_temp, 70.0);
        assert_eq!(args.off_temp, 50.0);
        assert_eq!(args.interval, 1000);
    }

    #[test]
    fn test_read_temp_mock() {
        // Simulate a temp string as if read from /sys/class/thermal/thermal_zone0/temp
        let input = "55000\n";
        let temp: i32 = input.trim().parse().unwrap();
        assert_eq!(temp, 55000);
    }

    #[test]
    fn test_fan_logic() {
        let mut fan_on = false;
        let on_temp = 60000;
        let off_temp = 45000;

        // Temp rises
        let temp = 61000;
        if !fan_on && temp >= on_temp {
            fan_on = true;
        }
        assert!(fan_on);

        // Temp drops
        let temp = 44000;
        if fan_on && temp <= off_temp {
            fan_on = false;
        }
        assert!(!fan_on);
    }
}

