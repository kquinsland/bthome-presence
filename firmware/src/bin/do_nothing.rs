#![no_std]
#![no_main]

//! An experiment to determine which parts of chip / peripheral config cost the most power.
//! Notes taken in line with features added / removed.

use defmt::{debug, info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::config::DcdcConfig;
use embassy_nrf::interrupt::{self, InterruptExt, Priority};
use embassy_time::Timer;
use nrf_softdevice::ble::advertisement_builder::{
    AdvertisementDataType, ExtendedAdvertisementBuilder, ExtendedAdvertisementPayload, Flag,
};
use nrf_softdevice::ble::{peripheral, TxPower};
use {defmt_rtt as _, panic_probe as _};

use nrf_softdevice::{raw, Softdevice};

// heapless str/vec for the pretend data
use arrayvec::{ArrayString, ArrayVec};

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = embassy_nrf::config::Config::default();

    //config.hfclk_source = HfclkSource::ExternalXtal;
    //config.lfclk_source = LfclkSource::ExternalXtal;
    config.dcdc = DcdcConfig { reg1: true };

    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    interrupt::SAADC.set_priority(Priority::P3);
    let mut p = embassy_nrf::init(config);
    debug!("embassy_nrf::init: done!");

    /*
       Goal here is to do nothing and see how power consumption goes.
       Not using any peripherals or soft device so the chip should be as dormant as possible.
       For all experiments, no changes to loop{} and always sampling 10Ksp/s for 20s.

       Simple as possible gets me
            1.54uA average, max of 19.98uA

        Enable JUST the DCDC gets me
            1.5uA average, max of 13.63uA

            Adding just gpiote_interrupt_priority = Priority::P2
            1.51uA average, max of 13.16uA

            Adding just time_interrupt_priority = Priority::P2
            1.51uA average, max of 14.08uA

            Adding just interrupt::SAADC.set_priority(Priority::P3)
            1.51uA average, max of 13.65uA

            Just for giggles, re-disabled the DCDC and got
            1.54uA average, max of 20.83uA

            Re-enabled it and we're back to
            1.51uA average, max of 13.69uA

        Trying just set up / start soft-device with no adverts
            1.15uA average, max of 14.67uA
        And with calling run() on it
            1.16uA average, max of 14.61uA

        And with adverts
            27.88uA average, max of 8.53uA

        Then adding bytes to the adverts
            35.04uA average, max of 8.68uA


        Setting just the hf clock source gets me
            179.07 uA average / max 302.37 ua

        Setting just the hf/lf clock sources gets me
            178.73uA average / max 302.5 ua

        Setting just the hf/lf clock sources and the DDC config gets me
            348.78uA average / max 406.15 ua

    */

    let sd_config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            // There is an external oscillator...
            source: raw::NRF_CLOCK_LF_SRC_XTAL as u8,
            // ctiv values are only to be set to non-zero values when using the RC oscillator
            rc_ctiv: 0,
            rc_temp_ctiv: 0,
            // Default is 250 ppm; I don't have a datasheet for the crystal so I'm just going with the default?
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_250_PPM as u8,
        }),

        // We are expecting ZERO connections but the crate requires this be at least one
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            // We want 0 connections but setting this value to 0 produces a panic:
            //  ERROR panicked at 'sd_ble_cfg_set 32 err InvalidParam'
            conn_count: 1,

            // Count of 1.25ms units set aside for connection
            // This can be tweaked for more/less throughput?
            event_length: 24,
        }),

        // We're trying to conserve as much ram as possible; we do not need a huge attributes table as we're just broadcasting
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: raw::BLE_GATTS_ATTR_TAB_SIZE_MIN,
        }),

        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 0,
            periph_role_count: 0,
        }),

        ..Default::default()
    };
    debug!("sd_config: made!");

    let sd = Softdevice::enable(&sd_config);
    debug!("Softdevice: enabled...");
    unwrap!(spawner.spawn(softdevice_task(sd)));
    debug!("Softdevice: running...");

    let mut config = peripheral::Config {
        ..Default::default()
    };

    let mac_addr = nrf_softdevice::ble::get_address(sd).bytes();
    let mut device_name = ArrayString::<10>::from("BTHPT_").unwrap();

    device_name.push(byte_to_hex(mac_addr[1])[0]);
    device_name.push(byte_to_hex(mac_addr[1])[1]);
    device_name.push(byte_to_hex(mac_addr[0])[0]);
    device_name.push(byte_to_hex(mac_addr[0])[1]);

    // For now it's still hard-coded data but at least we're dynamically adding to it?
    let mut bt_home_adv_data = ArrayVec::<_, 14>::new();

    // The shelly UUID + BT-Home version/flag byte
    for i in [0xd2, 0xfc, 0x40] {
        bt_home_adv_data.push(i);
    }

    // battery as a percentage
    bt_home_adv_data.push(0x01);
    bt_home_adv_data.push(0x61);

    // battery voltage
    bt_home_adv_data.push(0x0C);
    bt_home_adv_data.push(0x70);
    bt_home_adv_data.push(0x0b);

    // movement is not active
    bt_home_adv_data.push(0x22);
    bt_home_adv_data.push(0x01);

    // battery voltage
    bt_home_adv_data.push(0xF2);
    bt_home_adv_data.push(0x00);
    bt_home_adv_data.push(0x01);
    bt_home_adv_data.push(0x06);

    info!("Device name: {}", device_name.as_str());

    let ADV_DATA: ExtendedAdvertisementPayload = ExtendedAdvertisementBuilder::new()
        // See note above: we're sticking to BT 4.x (legacy) for now
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
        // Add the BT-Home data
        .raw(AdvertisementDataType::SERVICE_DATA_16, &bt_home_adv_data)
        .adapt_name("power_test")
        .build();

    let adv = peripheral::NonconnectableAdvertisement::NonscannableUndirected {
        adv_data: &ADV_DATA,
    };
    info!("Starting Advert task...");
    unwrap!(peripheral::advertise(sd, adv, &config).await);

    loop {
        debug!("doing nothing...");
        Timer::after_millis(2500).await;
    }
}

/// Converts a single nibble (4 bits) to a hexadecimal character.
fn nibble_to_hex_char(nibble: u8) -> char {
    match nibble {
        0x0..=0x9 => (b'0' + nibble) as char,
        0xa..=0xf => (b'a' + nibble - 10) as char,
        _ => '?', // Should never happen if used correctly
    }
}

/// Converts a byte to a two-character hexadecimal string.
fn byte_to_hex(byte: u8) -> [char; 2] {
    let high = (byte >> 4) & 0x0F;
    let low = byte & 0x0F;
    [nibble_to_hex_char(high), nibble_to_hex_char(low)]
}
