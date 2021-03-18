use crate::{ControllerExt, COLOR_OFF, LED_CHANNEL, LED_COUNT};
use anyhow::bail;
use byteorder::{BigEndian, ReadBytesExt};
use rs_ws281x::{Controller, RawColor};
use std::{
    io::{ErrorKind, Read},
    net::TcpStream,
};

pub trait Runner: Sized {
    fn new(socket: &mut TcpStream) -> anyhow::Result<Self>;

    fn run(&self, socket: TcpStream, controller: &mut Controller) -> anyhow::Result<()>;
}

pub struct ColorOnlyRunner {
    intensity: f32,
}

impl Runner for ColorOnlyRunner {
    fn new(socket: &mut TcpStream) -> anyhow::Result<Self> {
        let intensity = socket.read_f32::<BigEndian>()?;
        Ok(Self { intensity })
    }

    fn run(&self, mut socket: TcpStream, controller: &mut Controller) -> anyhow::Result<()> {
        let mut color = RawColor::default();
        loop {
            if socket.read(&mut color[..3])? == 0 {
                break;
            }
            controller.set_all_raw(color)?;
        }

        Ok(())
    }
}

pub struct IntensityOnlyRampRunner {
    color: RawColor,
}

impl Runner for IntensityOnlyRampRunner {
    fn new(socket: &mut TcpStream) -> anyhow::Result<Self> {
        let mut color = RawColor::default();
        socket.read(&mut color[..3])?;
        Ok(Self { color })
    }

    fn run(&self, mut socket: TcpStream, controller: &mut Controller) -> anyhow::Result<()> {
        loop {
            let mut intensity = match socket.read_f32::<BigEndian>() {
                Ok(i) => i,
                Err(err) if err.kind() == ErrorKind::UnexpectedEof => break,
                Err(err) => bail!(err),
            };

            intensity *= LED_COUNT as f32;

            for (i, led) in controller.leds_mut(LED_CHANNEL).iter_mut().enumerate() {
                if (i as f32) < intensity {
                    *led = self.color;
                } else {
                    *led = COLOR_OFF;
                }
            }
            controller.commit()?;
        }

        Ok(())
    }
}

pub struct ColorAndIntensityRampRunner {}