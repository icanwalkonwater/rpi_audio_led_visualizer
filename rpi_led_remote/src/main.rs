use crate::audio::AudioProcessor;
use anyhow::{anyhow, bail};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device,
};
use int_enum::IntEnum;
use rpi_led_common::{LedMode, MAGIC};
use std::{
    io::{stdin, stdout, Read, Stdout, Write},
    net::TcpStream,
};
use structopt::StructOpt;
use tui::{backend::CrosstermBackend, Terminal};

mod audio;

#[derive(Clone, Debug, StructOpt)]
struct Opt {
    /// Address to bind to
    address: String,

    /// A pattern to help take the right device
    /// Enabling this means disabling the manual selection of device
    #[structopt(short, long)]
    device_pattern: Option<String>,

    #[structopt(long)]
    only_color: bool,

    #[structopt(long)]
    only_intensity: bool,

    #[structopt(long)]
    color_and_intensity: bool,
}

fn main() -> anyhow::Result<()> {
    let opt: Opt = Opt::from_args();

    // Mode
    let mode = match (opt.only_color, opt.only_intensity) {
        (true, false) => LedMode::OnlyColor,
        (false, true) => LedMode::OnlyIntensity,
        (false, false) => bail!("You must choose a mode !"),
        _ => bail!("Only one mode can be active at a time !"),
    };
    println!("Mode selected: {:?}", mode);

    // Audio stuff
    let device = get_device(opt.device_pattern.as_ref().map(|s| s as &str).unwrap_or(""))?;
    println!("Found audio device {}", device.name()?);

    let config = device.default_input_config()?;

    // Setup socket
    let mut socket = TcpStream::connect(opt.address)?;
    println!("Connected to {}", socket.peer_addr()?);

    // Hello
    let magic = socket.read_u8()?;
    assert_eq!(magic, MAGIC);

    // Mode
    socket.write_u8(mode.int_value())?;

    match mode {
        LedMode::OnlyColor => {
            socket.write_f32::<BigEndian>(1.0)?;
        }
        LedMode::OnlyIntensity => socket.write_all(&[255, 0, 0])?,
        _ => todo!(),
    }

    // Audio processor
    let mut processor = AudioProcessor::new(100_000);
    let reader = device.build_input_stream(
        &config.into(),
        move |data: &[i16], _| {
            let intensity = processor.update(data);

            match mode {
                LedMode::OnlyColor => {
                    socket.write_all(&[255, 0, 0]).unwrap();
                }
                LedMode::OnlyIntensity => {
                    println!("Write intensity: {}", intensity);
                    socket.write_f32::<BigEndian>(intensity).unwrap();
                }
                _ => todo!(),
            }

            socket.flush().unwrap();
        },
        |e| {
            eprintln!("CPL Error: {:?}", e);
        },
    )?;

    // Start recording
    reader.play()?;

    println!("Recording...");
    stdin().read(&mut [0])?;

    reader.pause()?;

    Ok(())
}

fn setup_tui() -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
    let stdout = stdout();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn get_device(hint: &str) -> anyhow::Result<Device> {
    let host = cpal::default_host();

    let device = if hint.is_empty() {
        host.default_input_device()
            .ok_or(anyhow!("No default device found"))?
    } else {
        host.input_devices()?
            .find(|device| device.name().unwrap().contains(hint))
            .ok_or(anyhow!("Can't find a device with the hint"))?
    };

    Ok(device)
}