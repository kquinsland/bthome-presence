#![no_std]
#![no_main]

//! A prototype; meant for the RAM constrained 810 tags.
//! Does not use timers to duty cycle the advertisement as we don't have the RAM for it.

#[path = "../common.rs"]
mod common;

use defmt::{info, *};
use embassy_executor::Spawner;
use nrf_softdevice::ble::advertisement_builder::{
    AdvertisementDataType, ExtendedAdvertisementBuilder, ExtendedAdvertisementPayload, Flag,
};
use nrf_softdevice::ble::{peripheral, TxPower};

use nrf_softdevice::{raw, Softdevice};

// heapless str/vec for the pretend data
use arrayvec::{ArrayString, ArrayVec};

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

// Attempt to include the nrf softdevice binary in the final binary.
const SOFTDEVICE_VAL: &[u8] = include_bytes!("../../nrf-soft-device/s112_nrf52_7.3.0.bin");

#[link_section = ".softdevice"]
#[no_mangle]
pub static SOFTDEVICE_BIN: [u8; SOFTDEVICE_VAL.len()] =
    *include_bytes!("../../nrf-soft-device/s112_nrf52_7.3.0.bin");

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Alive!");

    let config = nrf_softdevice::Config {
        // TODO: figure out if this is the appropriate clock config?
        // There is an external oscillator...
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),

        // We are expecting ZERO connections but the crate requires this be at least one
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            // We want 0 connections but setting this value to 0 produces a panic:
            //  ERROR panicked at 'sd_ble_cfg_set 32 err InvalidParam'
            conn_count: 1,

            // Count of 1.25ms units set aside for connection
            // This can be tweaked for more/less throughput?
            // Changing this value doesn't appear to change the required RAM size.
            event_length: 32,
        }),

        // We're trying to conserve as much ram as possible; we do not need a huge attribtes table as we're just broadcasting
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: raw::BLE_GATTS_ATTR_TAB_SIZE_MIN,
        }),

        // Likewise, explicitly set the number of connections to 0
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 0,
            periph_role_count: 0,
        }),

        ..Default::default()
    };
    info!("Config made");

    let sd = Softdevice::enable(&config);
    info!("SoftDevice enabled...");
    unwrap!(spawner.spawn(softdevice_task(sd)));
    info!("SoftDevice running...");

    // See docs (//TODO: link) for the rationale behind these settings
    // There's a tradeoff between power consumption and responsiveness and range.
    let mut config = peripheral::Config {
        // How often the advertisement packet is sent out
        // Unit is in 0.625ms increments. Default is 400 => 250ms

        // 5 seconds is 8000 units of 0.625ms
        // 30 seconds is 48000 units of 0.625ms
        // TODO: make my own enum for this?
        // Values too big will cause a panic
        // sd_ble_gap_adv_set_configure err InvalidParam
        // Absolute max is 16384, apparently.
        // 16384 * 0.625ms = 10.24 seconds
        interval: 16384,

        // Likewise, we can tune the power consumption
        // 0dBm is the default and results in a peak current draw of ~20ma
        // Minus40dBm is the lowest power setting and results in a peak current draw of ~15ma
        tx_power: TxPower::Minus4dBm,
        ..Default::default()
    };

    debug!("Config interval set to {}", config.interval);

    /*

        impl Default for Config {
        fn default() -> Self {
            Self {
                primary_phy: Phy::M1,
                secondary_phy: Phy::M1,
                tx_power: TxPower::ZerodBm,
                timeout: None,
                max_events: None,
                interval: 400, // 250ms
                filter_policy: FilterPolicy::default(),
            }
        }
    }

         */

    /*
       BT 4.x supports 31 bytes of data in the advertisement packet and - if the scanning device actively inquires -
           another 31 bytes in the scan response packet.
       BT 5.x supports 255 bytes of data in the advertisement packet.

       It appears that the default behavior of the BT proxy component in ESPHome is to scan / connect to get the
           "second half" of the data but this does incur a small power penalty.
       The BTHome docs don't appear to really care if it's BT 4 or 5; they imply that the protocol can support _up to_ 255 bytes.

       So it appears that the limiting factor is hardware. Only some very new ESP32 chips support BT 5.0.

       So... I'm going to try as hard as I can to use _just_ legacy advertisement data.
       This should be compatible with the widest possible range of hardware devices and also the most power efficient.

        ------

       After some consideration, there's not a TON of value in letting the user set the name of the device.
       Doing this would require a lot of extra code and I don't know that I have the RAM for it.
       During some provisioning time, would need to set up a GATT service to allow the user to set the name.
       The name would also eat into the 31 bytes of data available in the advertisement packet.

       If absolutely required, user can always re-compile the firmware with the name hard-coded in.
       But a sane default is to allocate a few bytes to a semi-dynamic name.
           BTHPT_XXXX where XXXX is the last two bytes of the MAC address.
           BT Home Pet Tracker  . I may come re-visit this later
    */

    // Start with the MAC address
    // Note that endianness is reversed in the MAC address
    // If device address is e2:db:e8:62:67:0d, `mac` will be:
    //      RandomStatic:[0d, 67, 62, e8, db, e2]
    let mac_addr = nrf_softdevice::ble::get_address(sd).bytes();
    //debug!("mac_addr bytes: {=[u8]:08b}", mac_addr);
    debug!("mac_addr bytes: {=[u8]:02x}", mac_addr);

    // // BTHPT_XXXX format is 10 chars
    let mut device_name = ArrayString::<10>::from("BTHPT_").unwrap();

    // Oh how I miss std and format!()
    device_name.push(byte_to_hex(mac_addr[1])[0]);
    device_name.push(byte_to_hex(mac_addr[1])[1]);
    device_name.push(byte_to_hex(mac_addr[0])[0]);
    device_name.push(byte_to_hex(mac_addr[0])[1]);

    info!("Device name: {}", device_name.as_str());

    /*
      Details are in the notes but we have a grand total of 31 bytes to work with.
      This includes the type/length byte for each record in the payload.
      We must have the basic type `0x01` record with length of 2. (3 bytes total)
      However long the name is, we need to add 2 bytes for the type and length indicators
      Whatever room is left over is usable for the BT-Home data. This record also has 2 bytes over overhead.
      So that's 5 bytes of overhead.
      If the name is 10 bytes long, that's 12 bytes total.
       So we have 31 - 12 - 5 = 14 bytes to work with for the BT-Home data.

       let name_record_len = device_name.len() + 2;
       let bt_home_data_len = 31 - (name_record_len + 5);
       debug!("name_record_len: {}", name_record_len);
       debug!("ADU bytes for BT-HomeData: {}", bt_home_data_len);
    */

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

    // movenet is not active
    bt_home_adv_data.push(0x22);
    bt_home_adv_data.push(0x01);

    // battery voltage
    bt_home_adv_data.push(0xF2);
    bt_home_adv_data.push(0x00);
    bt_home_adv_data.push(0x01);
    bt_home_adv_data.push(0x06);

    debug!(
        "bt_home_adv_data ({}) : {=[u8]:02x}",
        bt_home_adv_data.len(),
        bt_home_adv_data.as_slice()
    );

    let ADV_DATA: ExtendedAdvertisementPayload = ExtendedAdvertisementBuilder::new()
        // See note above: we're sticking to BT 4.x (legacy) for now
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
        // Add the BT-Home data
        .raw(AdvertisementDataType::SERVICE_DATA_16, &bt_home_adv_data)
        // Whatever room is left over is for the name
        .adapt_name(&device_name)
        .build();

    // Trying to dump the bytes here so I can compare with the 'raw' shown in nrfConnect
    info!(
        "ADV_DATA({}): {=[u8]:02x}",
        ADV_DATA.len(),
        ADV_DATA.as_ref()
    );

    let adv = peripheral::NonconnectableAdvertisement::NonscannableUndirected {
        adv_data: &ADV_DATA,
    };
    info!("Starting Advert task...");
    defmt::println!("{:?}", SOFTDEVICE_BIN.len());
    unwrap!(peripheral::advertise(sd, adv, &config).await);
}

// /// Converts a single nibble (4 bits) to a hexadecimal character.
fn nibble_to_hex_char(nibble: u8) -> char {
    match nibble {
        0x0..=0x9 => (b'0' + nibble) as char,
        0xa..=0xf => (b'a' + nibble - 10) as char,
        _ => '?', // Should never happen if used correctly
    }
}

// /// Converts a byte to a two-character hexadecimal string.
fn byte_to_hex(byte: u8) -> [char; 2] {
    let high = (byte >> 4) & 0x0F;
    let low = byte & 0x0F;
    [nibble_to_hex_char(high), nibble_to_hex_char(low)]
}
