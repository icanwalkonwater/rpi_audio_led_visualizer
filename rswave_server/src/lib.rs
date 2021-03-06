use anyhow::anyhow;
use std::str::FromStr;
use structopt::StructOpt;

pub mod app;
pub mod led_controllers;
pub mod net;
pub mod runners;

#[derive(Copy, Clone, Debug, StructOpt)]
pub struct Opt {
    /// Port to use.
    #[structopt(short, long, default_value = "20200")]
    pub port: u16,

    /// Set overall brightness.
    #[structopt(short, long, default_value = "255")]
    pub brightness: u8,

    /// Reset the LED strip and exit.
    #[structopt(short, long)]
    pub reset: bool,

    /// Led strip type, will default to WS2811.
    /// Possible values: ws2811, gpio.
    #[structopt(short, long, default_value = "ws2811")]
    pub led_type: LedStripType,

    /// Amount of LEDs on the strip (only used with an addressable strip).
    #[structopt(short = "c", long, required_if("led_type", "ws2811"))]
    pub led_count: Option<usize>,

    /// Frequency in Hz to use for the PWM pins, only used with GPIO led type.
    #[structopt(long, default_value = "100.0", required_if("led_type", "gpio"))]
    pub pwm_freq: f64,

    /// The GPIO pin to use for the red when in GPIO led type.
    #[structopt(long, default_value = "23", required_if("led_type", "gpio"))]
    pub pin_red: u8,

    /// The GPIO pin to use for the green when in GPIO led type.
    #[structopt(long, default_value = "24", required_if("led_type", "gpio"))]
    pub pin_green: u8,

    /// The GPIO pin to use for the blue when in GPIO led type.
    #[structopt(long, default_value = "25", required_if("led_type", "gpio"))]
    pub pin_blue: u8,

    /// Delay during LED updates in milliseconds.
    #[structopt(long, default_value = "10")]
    pub led_update_period: u64,

    /// Controls the speed of the rainbow during the standby mode.
    #[structopt(long, default_value = "1.0")]
    pub standby_speed: f32,

    /// Reverse the rainbow effect of the standby runner.
    /// This effect will only be visible on addressable LED strips.
    #[structopt(long)]
    pub standby_reverse: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum LedStripType {
    Ws2811,
    Gpio,
}

impl FromStr for LedStripType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ws2811" => Ok(Self::Ws2811),
            "gpio" => Ok(Self::Gpio),
            _ => Err(anyhow!("Unknown led strip type !")),
        }
    }
}
