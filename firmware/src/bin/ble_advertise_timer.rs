#![no_std]
#![no_main]

//! This is the "main" application firmware.
//! It advertises battery level and presence information over BLE in BTHome format.
//! There are quite a few opportunities for optimization and refactoring in this code!

use core::mem;

#[path = "../common.rs"]
mod common;

use common::util::encoding::byte_to_hex;

use defmt::{info, *};
use embassy_executor::Spawner;
use embassy_nrf::interrupt::{self, InterruptExt, Priority};
use embassy_nrf::saadc::{ChannelConfig, Config, Resolution, Saadc, VddInput};
use embassy_nrf::{bind_interrupts, saadc};

use embassy_nrf::config::DcdcConfig;

use embassy_time::{with_timeout, Duration, Timer};
use nrf_softdevice::ble::advertisement_builder::{
    AdvertisementDataType, ExtendedAdvertisementBuilder, ExtendedAdvertisementPayload, Flag,
};

use nrf_softdevice::ble::peripheral;
use nrf_softdevice::{raw, Softdevice};

use arrayvec::{ArrayString, ArrayVec};

bind_interrupts!(struct Irqs {
    SAADC => saadc::InterruptHandler;
});

// Attempt to include the nrf softdevice binary in the final binary.
// We are declaring twice because the static needs to have a size associated with it and
// a const lets us get the size of the binary file at compile time.
#[cfg(feature = "with-softdevice")]
const SOFTDEVICE_VAL: &[u8] = include_bytes!("../../nrf-soft-device/s112_nrf52_7.3.0.bin");

#[cfg(feature = "with-softdevice")]
#[link_section = ".softdevice"]
#[no_mangle]
pub static SOFTDEVICE_BIN: [u8; SOFTDEVICE_VAL.len()] =
    *include_bytes!("../../nrf-soft-device/s112_nrf52_7.3.0.bin");

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

async fn do_advert(sd: &'static Softdevice, advertisement_data: ExtendedAdvertisementPayload) {
    loop {
        let phy_config = peripheral::Config {
            // Time to wait between advertising packets.
            // SoftDevices does not like values higher than 16384 (10.24 seconds)
            // For this particular application, power savings is way more important than
            // responsiveness.
            // Sending out advert every 6 seconds is fine; at least one of those is going to be picked up.
            // 9600 *.625ms = 6 seconds
            interval: 9600,

            // Likewise, we can tune the power consumption
            // 0dBm is the default and results in a peak current draw of ~20ma with pretty good range.
            // Minus40dBm is the lowest power setting and results in a peak current draw of ~15ma
            //  but a noticeable decrease in range.
            // Eventually both interval and power level will be configurable via app.
            ..Default::default()
        };

        let advert_payload = peripheral::NonconnectableAdvertisement::NonscannableUndirected {
            adv_data: &advertisement_data,
        };
        info!("do_advert: advertising...");
        info!(
            "do_advert: advertisement_data({}): {=[u8]:02x}",
            advertisement_data.len(),
            advertisement_data.as_ref()
        );
        unwrap!(peripheral::advertise(sd, advert_payload, &phy_config).await);
        // Should never get here...
        info!("do_advert: stop advertising...");
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Might be worth doing a bit more work in GHA to build a more informative version string with
    // the branch or tag name instead of just the short hash.
    info!("Main is alive! Build:{}", env!("CARGO_PKG_VERSION"));
    let mut config = embassy_nrf::config::Config::default();

    // Enable the DCDC converter for (slightly) lower power consumption
    config.dcdc = DcdcConfig { reg1: true };

    // Init required for timers but we need to adjust their priority to not break the softdevice
    // Softdevice has reserved priorities 0, 1 and 4 and will cause a panic if we try to use them for other things
    //      panicked at 'sd_softdevice_enable err SdmIncorrectInterruptConfiguration'
    // See: https://github.com/embassy-rs/nrf-softdevice?tab=readme-ov-file#troubleshooting
    // 0 is Highest. Lower priority number can preempt higher priority number

    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    interrupt::SAADC.set_priority(Priority::P3);

    let mut p = embassy_nrf::init(config);
    debug!("embassy_nrf::init: done!");

    let sd_config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_XTAL as u8,
            // ctiv values are only to be set to non-zero values when using an RC oscillator
            // Both the HolyIoT and Duo tags use an external crystal
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

    // TODO: what happens in HA when we omit the device name from some of the packets?
    // I suspect that the sudden absence of a name will not trigger a rename in the UI but
    // it'll be good to confirm this. Assuming this is true, I can pack more information per
    // advert at the expense of the user not reliably seeing a device name when they use their
    // phone to scan for devices... which is not expected to happen often!

    // Start with the MAC address
    // Note that endianness is reversed in the MAC address
    // If device address is e2:db:e8:62:67:0d, `mac` will be:
    //      RandomStatic:[0d, 67, 62, e8, db, e2]
    let mac_addr = nrf_softdevice::ble::get_address(sd).bytes();
    debug!("mac_addr bytes: {=[u8]:02x}", mac_addr);

    // BTHPT_XXXX format is 10 chars
    let mut device_name = ArrayString::<10>::from("BTHPT_").unwrap();

    device_name.push(byte_to_hex(mac_addr[1])[0]);
    device_name.push(byte_to_hex(mac_addr[1])[1]);
    device_name.push(byte_to_hex(mac_addr[0])[0]);
    device_name.push(byte_to_hex(mac_addr[0])[1]);
    info!("Device name: {}", device_name.as_str());

    // Room for BT-Home payload
    let mut bt_home_adv_data = ArrayVec::<u8, 16>::new();

    // The shelly UUID + BT-Home version/flag byte
    // TODO: probably make a small module with CONSTs for the values
    for i in [0xd2, 0xfc, 0x40] {
        bt_home_adv_data.push(i);
    }

    // Next push the packet ID into the payload
    let mut packet_id = 0 as u8;
    // TODO: create constants module so we have human readable names for types
    // e.g. BT_HOME_DATA_TYPE_PACKET_ID = 0x00
    bt_home_adv_data.push(0x00);

    // Longer term, I will probably need to re-factor this code to change _what_ is advertised
    // each interval; there isn't enough room in _one_ advertisement payload to send
    // battery and movement and packet ID and device_id/firmware_version ... etc.
    // See: https://bthome.io/format/#misc-data
    // Right now we're just mentally keeping track that the 4th byte is the packet ID
    // It might be worth creating a higher-level struct to wrap the underlying raw bytes and abstract this away a bit
    bt_home_adv_data.push(packet_id as u8);

    // Object ID 0x01 => Battery, 1 bytes
    bt_home_adv_data.push(0x01);
    // Push a bogus value now, will update once we actually poll ADC
    bt_home_adv_data.push(0xff);

    // Going to try also broadcasting a bool "presence" value to see if this allows me to ditch
    // the manual / template automation that I _was_ using to link the RSSI to device_tracker / person.
    // 0x25 => Presence, bool.
    bt_home_adv_data.push(0x25);
    // We hard-code "home" because any time the device is advertising, it's at home.
    bt_home_adv_data.push(0x01);

    loop {
        // New advertise interval starting up, push the correct packet_id
        bt_home_adv_data.push(packet_id as u8);
        bt_home_adv_data.swap_remove(4);

        // TODO: this whole thing should be refactored into a separate function
        // Following the pattern here: https://github.com/embassy-rs/embassy/blob/main/examples/nrf52840/src/bin/twim_lowpower.rs
        // If I drop the ADC at the end of the loop / before sleep... will we have lower power usage. In testing, ~ 2uA less power usage!
        let mut adc_config = Config::default();
        adc_config.resolution = Resolution::_10BIT;

        let channel_config = ChannelConfig::single_ended(VddInput);
        let mut saadc = Saadc::new(&mut p.SAADC, Irqs, adc_config, [channel_config]);
        saadc.calibrate().await;
        debug!("adc: calibrated!");

        // Read the battery
        let mut buf = [0; 1];
        saadc.sample(&mut buf).await;

        // Drop the ADC to save (a tiny amount of) power
        mem::drop(saadc);

        // Turn the 10 bit value into a float
        let sample = f32::from(buf[0]);
        let voltage = (sample / 1024.0) * 3.6;

        // Percentage of _usable_ voltage range
        let percentage = ((voltage - 1.7) / (3.6 - 1.7)) * 100.0;
        info!(
            "sample: {} | voltage: {=f32:02} | percentage: {=f32:02}",
            sample, voltage, percentage
        );

        bt_home_adv_data.push(percentage as u8);
        // Again, not super happy to have to mentally keep track that the 7th byte
        // is where we store the battery; this should be re-factored into a struct :)
        bt_home_adv_data.swap_remove(6);

        debug!(
            "bt_home_adv_data ({}) : {=[u8]:02x}",
            bt_home_adv_data.len(),
            bt_home_adv_data.as_slice()
        );

        let advertisement_data: ExtendedAdvertisementPayload = ExtendedAdvertisementBuilder::new()
            .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
            // Add the BT-Home data
            .raw(AdvertisementDataType::SERVICE_DATA_16, &bt_home_adv_data)
            .adapt_name(&device_name)
            .build();

        let res = with_timeout(Duration::from_secs(10), do_advert(sd, advertisement_data)).await;
        // should result in Err(TimeoutError)
        debug!("advert time for {} elapsed: {:?}", packet_id, res);
        // Increment the packet ID
        packet_id = packet_id.wrapping_add(1);

        // Advertising should have stopped, attempt to enter a low power state
        info!("Stopping advertising for a moment");
        Timer::after(Duration::from_secs(10)).await;
    }
    // TODO: use WDT to recover from panics?
    // Yes, a lot of work went into not panicking but it would be nice to reboot
    //  in the event that we do panic.

    // TODO: pull firmware version from cargo.toml / git tags and transmit that w/ BTHome data?
}
