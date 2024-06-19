#![no_std]
#![no_main]

//! Simple timer example for testing power consumption and sleep modes.

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _ = embassy_nrf::init(Default::default());

    info!("Hello World!");

    loop {
        info!("high");
        Timer::after_millis(300).await;
        info!("low");
        Timer::after_secs(2).await;
    }
}
