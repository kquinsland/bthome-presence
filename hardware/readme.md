<!-- omit from toc -->
# Supported Hardware

Ali Express is _full_ of cheap nr52 based BT LE tags.
Once you filter out all the generic "reseller" stores, you're left with two main manufacturers:

- [HolyIoT](www.holyiot.com)
- [DUOWEISI](www.radioland-china.com)

Neither of them have a particularly clear and easy to navigate AliExpress store page but I've linked directly to the tags I'd suggest using.

Really, any nRF52 series chip should work.

The main difference between the chips is the amount of RAM they have.

In testing, the `810` chip can barely handle the basic functionality of this project and the `832` chip has plenty of room for the code to handle additional functionality and sensors.

| nRF chip | ram      | rom        | notes                                            |
| -------- | -------- | ---------- | ------------------------------------------------ |
| 52810    | 24k      | 192k       | BARELY enough ram for "broadcast only" function  |
| 52820    | 32k      | 256k       | untested, should work just as the 832 does       |
| 52832    | 64/32 KB | 512/256 KB | enough ram for additional sensors                |
| 52840    | 256 KB   | 1 MB       | untested; probably overkill for this application |

## Accessories

In some cases, I've included some "accessories" that I've designed to make working with the tags easier.

### pogo programmer

A 3d printable jig that makes it easier to program/debug the tags w/o soldering wires to the programming pads.

### AirTag form factor adapter

Since the cheap tags are _just_ PCBs, they can rattle around inside of the cavities designed for Apple AirTags.
For some of the tags, I've designed a 3d printable enclosure that makes them fit snugly instead of rattling around.

## HolyIoT

Model [21014](https://www.aliexpress.us/item/3256804085831056.html) is a `810` based tag.
I do not have a pogo programmer for this tag.
The [datasheet](./holy-iot/21014/HOLYIOT-21014-nRF52810%20datasheet%20.pdf) does have a mechanical drawing of the programming pads so it should be relatively easy to make one.

I do have a air-tag form factor ["adapter" for this tag](./airtag-adapter/HolyIoT_21014_airtag-adapter.step).

The `22040` tag is another `810` based tag but due to it's rectangular shape, it's not a good candidate for the air-tag form factor adapter.

I do have a [pogo programmer](./holy-iot/22040/programmer/readme.md) for this model.

## DUOWEISI

Duoweisi does not seem to assign a unique SKU/Model number to each hardware variation, unfortunately.

Despite the similar hardware, the PCB layouts are different enough that the pogo programmers have to be custom designed for each model/variant 😠!

Order the `LS2DH` variant of the tag [here](https://www.aliexpress.us/item/3256805027170746.html). I **do not** have any accessories for the non-`LS2DH` variant of the tag!

I do have a pogo [programmer](./DUOWEISI/programmmer/readme.md) for this variant.

I do have an air-tag form factor ["adapter" for this tag](./airtag-adapter/DUOWEISI_NRF52832_LS2DH_airtag-adapter.step).
