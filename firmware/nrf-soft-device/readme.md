<!-- omit from toc -->
# Flashing nrf SoftDevice

Both the HolyIoT and Duoweisi tags ship with an old version of the nrf soft device; something in the 6.x range.
Embassy/nrf-soft-device requires 7.x or later.

Later versions of 7.x do come with _slightly_ higher flash/ram usage so using the latest version with the `810` chip is not always the best idea.
As of writing, the latest supported version is `7.3.0` of [`S122`](https://www.nordicsemi.com/Products/Development-software/S112/Download?lang=en#infotabs)

Flashing just the soft-device is easy:

```shell
❯ cd nrf-soft-device/s112_nrf52_7.3.0
❯ probe-rs erase --chip nrf52810_xxAA
❯ probe-rs download --verify --format hex --chip nrf52810_xxAA s112_nrf52_7.3.0_softdevice.hex
<...>
Finished in 6.545s
```

With the soft-device flashed, the application can then be flashed as normal.

## Prepping soft device for bundled firmware distribution

During development, it's helpful to flash the main application independently from the softdevice.
This is not adventageous in production.

**The soft-device binary is subject to the nordic license agreement** which can be read [here](./LICENSE).

After confirming that a particular build/version of softdevice works with the application, it is possible to consolidate the soft device and application into a single `bin` file.
This is done at compile/link time by some additional directives in the `memory-${variant}.x` files and with a compile time feature flag.

First, convert the `hex` file to a `bin` file:

```shell
❯ cd hex_tools
❯ ./hex2bin.py ../s112_nrf52_7.3.0/s112_nrf52_7.3.0_softdevice.hex ../s112_nrf52_7.3.0.bin
```

Then update the application source with the correct path to the softdevice binary:

```rust
#[cfg(feature = "with-softdevice")]
const SOFTDEVICE_VAL: &[u8] = include_bytes!("../../nrf-soft-device/s112_nrf52_7.3.0.bin");

#[cfg(feature = "with-softdevice")]
#[link_section = ".softdevice"]
#[no_mangle]
pub static SOFTDEVICE_BIN: [u8; SOFTDEVICE_VAL.len()] =
    *include_bytes!("../../nrf-soft-device/s112_nrf52_7.3.0.bin");
```

Then run the application with the `with-softdevice` feature:

```shell
❯ cargo run --bin ble_advertise_timer --features nrf52832 --features with-softdevice --release
<...>
```

Or build and flash in two steps:

```shell
❯ cargo build --bin ble_advertise_timer --features nrf52832 --features with-softdevice --release
❯ probe-rs run --chip nRF52832_xxAA --disable-progressbars firmware/target/thumbv7em-none-eabihf/release/ble_advertise_timer
```
