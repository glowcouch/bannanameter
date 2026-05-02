//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{self, Input, Pull},
    i2c::{Config, I2c},
};
use embassy_time::{Delay, Timer};
use embedded_hal_bus::i2c::RefCellDevice;
use gpio::{Level, Output};
use hcsr04::{Hcsr04, NoTemperatureCompensation};
use hd44780_driver::HD44780;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C0_IRQ => embassy_rp::i2c::InterruptHandler<embassy_rp::peripherals::I2C0>;
});

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

    let sda = p.PIN_8;
    let scl = p.PIN_9;

    let i2c = I2c::new_async(p.I2C0, scl, sda, Irqs, Config::default());
    let mut lcd = HD44780::new_i2c(i2c, 0x3F, &mut embassy_time::Delay).unwrap();

    lcd.clear(&mut embassy_time::Delay).unwrap();
    lcd.reset(&mut embassy_time::Delay).unwrap();
    lcd.set_cursor_pos(0x40, &mut embassy_time::Delay).unwrap();
    lcd.write_str("hello", &mut embassy_time::Delay).unwrap();

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
