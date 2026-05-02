//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

use core::{fmt::Write, str::FromStr};
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{self, Input, Pull},
    i2c::{Config, I2c},
};
use embassy_time::{Delay, Timer};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::{delay::DelayNs, digital::Wait};
use gpio::{Level, Output};
use hcsr04::{Hcsr04, NoTemperatureCompensation};
use hd44780_driver::{bus::DataBus, HD44780};
use heapless::String;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C0_IRQ => embassy_rp::i2c::InterruptHandler<embassy_rp::peripherals::I2C0>;
});

struct Device<A, B, C> {
    distance: Hcsr04<A, B, C>,
}

struct Screen<const N: usize> {
    title: String<N>,
    value: f64,
}

impl<const N: usize> Screen<N> {
    fn draw<B: DataBus>(&self, lcd: &mut HD44780<B>) {
        lcd.clear(&mut embassy_time::Delay).unwrap();
        lcd.reset(&mut embassy_time::Delay).unwrap();

        lcd.set_cursor_pos(0x40, &mut embassy_time::Delay).unwrap();
        lcd.write_str(&self.title, &mut embassy_time::Delay)
            .unwrap();

        let mut value_string: String<16> = String::new();
        let _ = write!(&mut value_string, "{:.3}", self.value); // ignore errors here

        lcd.set_cursor_pos(0x00, &mut embassy_time::Delay).unwrap();
        lcd.write_str(&value_string, &mut embassy_time::Delay)
            .unwrap();
    }
}

enum Modes {
    Distance,
}

impl Modes {
    async fn render<A: OutputPin, B: InputPin + Wait, C: DelayNs>(
        &self,
        device: &mut Device<A, B, C>,
    ) -> Screen<16> {
        match self {
            Modes::Distance => {
                let distance = device.distance.measure_distance().await.unwrap();

                Screen {
                    title: String::from_str("distance").unwrap(),
                    value: distance as f64,
                }
            }
        }
    }

    fn next(&self) -> Self {
        match self {
            Modes::Distance => Modes::Distance,
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let trig = Output::new(p.PIN_0, Level::Low);
    let echo = Input::new(p.PIN_1, Pull::None);

    let hcsr04 = Hcsr04::builder()
        .trig(trig)
        .echo(echo)
        .delay(Delay)
        .temperature(NoTemperatureCompensation)
        .build();

    let mut device = Device { distance: hcsr04 };

    let sda = p.PIN_8;
    let scl = p.PIN_9;

    let i2c = I2c::new_async(p.I2C0, scl, sda, Irqs, Config::default());
    let mut lcd = HD44780::new_i2c(i2c, 0x3F, &mut embassy_time::Delay).unwrap();

    let mode_switch = Input::new(p.PIN_3, Pull::Up);
    let mut mode_switch_was_high = false;

    let mut mode = Modes::Distance;

    loop {
        mode.render(&mut device).await.draw(&mut lcd);

        // rotate mode when button pressed
        if mode_switch.is_high() && !mode_switch_was_high {
            mode = mode.next();
        }
        mode_switch_was_high = mode_switch.is_high();

        Timer::after_millis(10).await;
    }
}
