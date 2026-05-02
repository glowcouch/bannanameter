//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{self, Input, Pull};
use embassy_time::{Delay, Timer};
use gpio::{Level, Output};
use hcsr04::{Hcsr04, NoTemperatureCompensation};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut led = Output::new(p.PIN_25, Level::Low);

    let trig = Output::new(p.PIN_0, Level::Low);
    let echo = Input::new(p.PIN_1, Pull::None);

    let mut hcsr04 = Hcsr04::builder()
        .trig(trig)
        .echo(echo)
        .delay(Delay)
        .temperature(NoTemperatureCompensation)
        .build();

    led.set_high();

    loop {
        let dist = hcsr04.measure_distance().await.unwrap();

        if dist > 10. {
            led.set_low();
        } else {
            led.set_high();
        }

        Timer::after_millis(10).await;
    }
}
