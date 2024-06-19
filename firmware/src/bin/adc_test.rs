#![no_std]
#![no_main]

//! Testbed for ADC.
//! Used with ppk2 to confirm supported voltage ranges and tune
//! the raw ADC to voltage range calcs.

use defmt::{debug, info};
use embassy_executor::Spawner;
use embassy_nrf::saadc::{ChannelConfig, Config, Saadc, VddInput};
use embassy_nrf::{bind_interrupts, saadc};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SAADC => saadc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_p: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut config = Config::default();
    config.resolution = saadc::Resolution::_10BIT;

    // Defaults are Internal reference (.6v), Gain 1/6, Bypass resistor network
    // And I don't know what ... any of that means :P
    let channel_config = ChannelConfig::single_ended(VddInput);
    let mut saadc = Saadc::new(p.SAADC, Irqs, config, [channel_config]);
    saadc.calibrate().await;
    debug!("calibrated");

    loop {
        let mut buf = [0; 1];
        saadc.sample(&mut buf).await;
        /*
             When set to 10 bit adc will spit out a value from 0-1023.
             Datasheet says absolute max is 3.9 but nominal is 1.7-3.6v.
             So we should map 0-1023 to 0v-3.6v to get the voltage from the raw ADC value
        */
        // Turn the 10 bit value into a float
        let sample = f32::from(buf[0]);
        let voltage = (sample / 1024.0) * 3.6;

        // Percentage of _usable_ voltage range
        let percentage = ((voltage - 1.7) / (3.6 - 1.7)) * 100.0;

        info!(
            "sample: {} | voltage: {=f32:02} | percentage: {=f32:02}",
            sample, percentage, voltage
        );
        Timer::after_millis(1000).await;
    }
}
