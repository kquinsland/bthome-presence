
<!-- omit from toc -->
# BTHome compatible presence detection tag firmware

This is an educational project to learn a bit more about [BT LE](https://en.wikipedia.org/wiki/Bluetooth_Low_Energy) and embedded Rust development with [embassy](https://embassy.dev/).

The primary goal is to create an alternative / [BTHome](https://bthome.io/) compatible firmware for presence detection that "just works" with Home Assistant.
The firmware targets the cheap nrf52 based BT LE tags that are available on Ali Express; they come in a variety of form factors and price points.
For my needs, this firmware will be loaded onto an "airtag" form factor tag that is attached to my dog's collar.

- [Background](#background)
- [Requirements](#requirements)
  - [Battery life](#battery-life)
  - [Home Assistant integration](#home-assistant-integration)
  - [Extra features](#extra-features)
- [Make one](#make-one)
- [Hardware](#hardware)
- [TODO](#todo)

## Background

I have several Home Assistant automations that change behavior based on how empty/occupied my home is.

E.G.:

- if _nobody_ is home, set the thermostat to a different setting compared to if just _I_ am home or just _the dog_ is home...etc.
- depending on who's home, [send the roomba to clean different areas](https://github.com/Hypfer/Valetudo).

Using basic BT LE tags for presence detection isn't a new idea, but they all have some drawbacks:

- AirTags are well made and have virtually global coverage but have no official API; there is no good way to programmatically get the location of an AirTag with Home Assistant.
- Other commercial solutions either have no API or come with a subscription fee or some other deal breaker like no user serviceable battery.
- The cheap tags on Ali Express have variable build quality, no global network and no standard broadcast format. Depending on which tag/firmware, surfacing battery state in Home Assistant [requires some work](https://old.reddit.com/r/homeassistant/comments/133l3ba/reading_battery_data_from_holyiot_beacons_using/) is possible with some work.

## Requirements

I just need a simple "home/not-home" indicator that works with Home Assistant and has a reasonable battery life.
While not a strict "must have", surfacing the battery level of the tag in Home Assistant in the most power-efficient way possible is a key differentiator compared to the stock firmware that ships with the cheap tags.

TL;DR, presence detection device must:

- Have at least 6 months battery life in nominal conditions
- Work out of the box with Home Assistant

For a small additional cost, the super cheap tags from Ali Express come equipped with additional sensors like temperature, humidity, light, acceleration, etc.
These are nice to have but not strictly necessary for my core use case; support for these sensors is planned but not a priority.

### Battery life

BT LE is already pretty power-efficient but there's only so much capacity in a typical [CR2032](https://en.wikipedia.org/?title=CR2032_battery&redirect=no) coin cell battery.

I already have a bi-annually task to replace the batteries in various other CR2032 powered devices around the house, so **I'd like to keep the battery life of this tag to at least 6 months**.

### Home Assistant integration

When [equipped with a suitable radio](https://www.home-assistant.io/integrations/bluetooth/), Home Assistant can interact with BT LE devices.
I have a few of [these](https://www.printables.com/model/522900-external-antenna-enclosure-for-lilygo-esp32-poe) running [BT-Proxy](https://esphome.io/components/bluetooth_proxy.html) distributed around my home for this purpose.

Despite being a ubiquitous technology, BT LE is not a standard protocol in the same way that [Z-Wave](https://en.wikipedia.org/wiki/Z-Wave) is!
Virtually every BT equipped device that HomeAssistant supports requires [glue code](https://github.com/search?q=repo%3Ahome-assistant%2Fcore%20%22dependencies%22%3A%20%5B%22bluetooth_adapters%22%5D%2C&type=code) that helps Home Assistant interpret the data coming from the device.

Fortunately, Home Assistant has co-developed a light weight protocol for BT LE devices: [BTHome](https://bthome.io/).
Any device that implements this protocol **automagically shows up in Home Assistant**; no custom glue code required.

### Extra features

Some of the tags I have come with additional sensors like temperature, humidity, acceleration, light ... etc.
BTHome does have support for a wide variety of sensors but they're not necessary for my core use case.
I have plans to add support for at least the accelerometer that often comes on these tags but no time line.

## Make one

The process isn't difficult, but do require at least a working ARM programmer and, in all likely hood, some basic soldering skills.
Some parts of the process can be skipped or shortened as needed:

- You don't need to print and build a pogo programmer if you're OK just soldering wires directly to the tag for programming
- You don't need a working dev environment if you're OK just flashing the pre-built firmware.

Outline of the process:

1. get a [supported](./hardware/readme.md#supported-hardware) tag.
   1. build a [programmer](./hardware/DUOWEISI/programmmer/readme.md) if desired.
2. [flash](./firmware/readme.md#flashing) the firmware onto the tag.
3. deploy / integrate with Home Assistant.

## Hardware

Is covered in depth in the [hardware](hardware/readme.md) section.

**TL;DR:** any nrf52 tag should work but for anything more than the absolute basic functionality, aim for more ram than the nrf52810 😃.

## TODO

- [ ] Get basic OTA firmware update working. For power savings, probably only want to start the server for the first few min after power up. Say a 5 min window to connect and update broadcast power, interval, device name, enable/disable the accelerometer and push the new firmware. Then go back to the normal NonConnectable broadcast / sleep cycle

- [ ] pre-commit hooks
- [ ] Fix `cargo test`
  - something breaks tests when you have `no_std` in the mix :/
