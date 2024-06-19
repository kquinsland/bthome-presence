<!-- omit from toc -->
# BT-Home Notes

Trying to figure out what a valid BT-Home advert payload looks like as well as how to generate one.

Starting with a device name of `BTH-PP`:

```text
nrf_raw: 0x02010607084254482D5050
ADV_DATA(11): [00000010, 00000001, 00000110, 00000111, 00001000, 01000010, 01010100, 01001000, 00101101, 01010000, 01010000]
```

And then adding one char to the name to make it `BTH-PPT`:

```text
nrf_raw: 0x02010608084254482D505054
ADV_DATA(12): [00000010, 00000001, 00000110, 00001000, 00001000, 01000010, 01010100, 01001000, 00101101, 01010000, 01010000, 01010100]
```

So adding one char to the name added one byte to the payload and we can see this in the hex representation of the payload as well.

```text
00 01 02 03 04 05 06 07 08 09 10 11
02 01 06 07 08 42 54 48 2D 50 50
02 01 06 08 08 42 54 48 2D 50 50 54
```

Only difference is on the `03` location; it went from `07` to `08`.
It's no coincidence that BTH-PP and BTH-PPT are 6 / 7 chars long (len + terminate byte) and this makes sense because B is `0x42` in hex and T is `0x54` in hex... etc.

So that's the name part of the payload.

Reading through the [PDF](Assigned_Numbers.pdf), we have TWO records.

First, len 02, type 01 (type is flags), data is 0x06 which decodes to 00000110 which has 1 set for both LE General DIscoverable Mode and BREDR Not Supported.
This seems to match the code:

```rust
    static ADV_DATA: ExtendedAdvertisementPayload = ExtendedAdvertisementBuilder::new()
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
        // in nrf connect, this is "shortened local name"
        .short_name("BTH-PPT")
        .build();
```

I understand where various ranges in the advertisement payload come from... now how do I add my own data to the payload?

## Working with BT-Home themo module

Using a `LYWSD03MMC` running the BT-Home firmware for a reference.
The reference device has a name of "ATC_D3DCEE".

I see a payload of:

```text
0x02 01 06 0E 16 D2 FC 40 00 00 01 64 02 BE 07 03 64 11 0B 09 41 54 43 5F 44 33 44 43 45 45
```

And the device is currently showing a temp of `0x64` (100 in decimal).

```text
List of 16-bit Service Data UUIDS: 0000fcd2-0000-1000-8000-00805f9b34fb (Allterco Robotics Itd)
```

From [this](https://www.argenox.com/library/bluetooth-low-energy/ble-advertising-primer/) article, we learn that the BT SIG has a standard UUID base.
The first 96 bits are fixed and the top most 32 bits are up to me?

So this means that I'm _really_ just dealing with `0000fcd2` and we _do_ see both `0xD2` and `0xFC` in the payload. The endianness is just different, though... so that's a bit confusing.

``` text
02 01 06            TYPE: 01 (Flags) 


0E 16 D2 FC 40 00 00 01 64 02 BE 07 03 64 11
(LEN: 0x0E [14], TYPE: 16 [service data; 16bit UUID])

So let's take a closer look at the actual data portion
D2 FC 40 00 00 01 64 02 BE 07 03 64 11

D2FC = 16 bit UUID; this is fixed and required

40 = BTHome Device Information  (0x40) -> 0b1000000
Note. bit 0 is the most right number, bit 7 is the most left number

    bit 0: 0 = False: No encryption
    bit 2: 0 = False: Device is sending regular data updates
    bit 5-7: 010 = 2: BTHome Version 2.

This then leaves us with 00 00 01 64 02 BE 07 03 64 11


00 00             => ??? This might be padding or some other data; not clear if needed or not?
01 64             => 01: battery 0x64 -> 100% (which is what I was seeing in HA ui)
02 BE 07          => 02: temperature 0xBE07 -> 1982 -> 19.82C (which is what I was seeing in HA ui)
03 64 11          => 03: humidity 0x6411 -> 4452 -> 44.52% (which is what I was seeing in HA ui)

(16bit conversion done with https://cryptii.com/pipes/integer-encoder)

0B 09 41 54 43 5F 44 33 44 43 45 45
(LEN: 0x0b, TYPE: 0x09 [name], DATA: ATC_D3DCEE)

```

But here's the question.
In HA, I have the following entities for the device

- Humidity            (percentage)
- Power                 (bool)
- Temp                  (Celsius)
- Voltage              (Volts)
- Battery               (percentage)
- Packet ID             (disabled by default; This explains the odd `0x00` object IDs in _some_ packet payloads)
- Signal Strength  (disabled by default; this is the only entity that's INFERRED)

In the decoded packet, I can account for Humidity, Temp, Battery percentage but not voltage.
This ... suggests? that there is more data being sent when ESPHome/BT_Proxy connects for the second half of the data?

Either that or multiple advertisements are being sent and a different set of measurements are being sent in each one?
The "ID" for low voltage is 0x0C so I should see that in a payload...

And as luck would have it, I open nRF connect and see this

```text
0x02 01 06 0B 16 D2 FC 40 00 C5 0C 70 0B 10 01 0B 09 41 54 43 5F 44 33 44 43 45 45

known: 02 01 06
known: 0B 09 41 54 43 5F 44 33 44 43 45 45

to figure out:
0B 16 D2 FC 40 00 C5 0C 70 0B 10 01 

0x0B = 11 bytes
0x16 = TYPE_16 bit UUID
0xD2FC = 16 bit UUID
0x40 = BTHome Device flags/version


00 C5              => 00: packet_id 0xC5 is the packet ID; This is optional; might consider adding if I have RAM to spare...

0C 70 0B           => 0C: voltage 0x700b -> 2928 -> 2.928V (which is what I was seeing in HA ui)
10 01              => 10: power (bool)  0x01 -> 1 -> True (Indicating that the device is ... on)

```

So. Of the 6 different entities that are reported in HA, I can account for all of them ... in _different_ payloads.
This more or less confirms that the way to deal with a limited payload is to just advertise multiple times with different data in each one.
This is more efficient than waiting to be scanned.

I do not know if the device name is required in all payloads or not. Might be worth testing to confirm that Home Assistant uses the sending MAC address to group entities together and that the name going missing from one payload doesn't cause the device to be "lost" in HA.

## What data do _i_ want to send?

I am limited to 31 bytes per advertisement packet.

Of those 31, several are already taken up by the BTHome protocol/flags.

- 8 bytes are fixed and can't be changed; This leaves me with 23 bytes to work with for data and name.
- Of those 23 bytes, 2 are the header for the name data
- Included in the 8 fixed bytes is the header for the data portion of the payload.

31 - 8 - 2 = 21 bytes for data. **Total**.

We get presence detection for free; no matter what the device is sending, if HA is receiving, the tag is close by.

The next most important bit of data is battery voltage so I can get proactive alerts when the battery is low.

After that, the additional sensors are nice to have but not critical.

### Battery

BTHome supports battery voltage AND percentage.
Raw voltage is more useful; percentage can be calculated on the receiving end if needed.
Percentage also depends on knowing the max/min voltage which is a function of battery chemistry.
Since all targeted tags use CR2032 tags, I can make some assumptions about min/max voltage but doo need to acknowledge that the voltage bands differ (slightly) for rechargeable vs non rechargeable cells.

Sending voltage with 1-2 decimal places is the most useful and allows for the simplest on-device software implementation.
Future revisions could include allowing configuration of the voltage range so that the percentage can be calculated on the device side rather than the receiver side with something like template sensors in HA.

### Movement

I can use the signal strength to estimate distance and I can measure changes in signal strength to infer movement.
It should be noted that this is not super accurate; the dog just rolling over to change nap positions could be the difference between the tag broadcasting directly to open air vs the dog lying on top of the tag.
From the received signal side, there would be a big difference in signal strength but the tag itself wouldn't have moved at all.

For this reason, I did "splurge" on one of the "premium" tags that has an accelerometer.
I didn't bother buying any of the "loaded" tags that come with many sensors because I don't need them and they're more expensive (in terms of $ **and** battery life).

The accelerometer should be optional, though:

- It's about $2/ tag _cheaper_ to omit.
- It's about 2uA cheaper to omit so there's a slightly longer battery life w/o it.
- Some use cases just need presence detection; movement detection is not needed.

While BTHome does support "raw" acceleration data, I don't think that's useful for this application given the battery life constraints.
The OEM firmware from HolyIoT only has a simple "has the tag moved in the last 20 seconds" sensor, presumably for similar reasons.

There may be some value in using the accelerometer's built in filter/interrupts to only wake up the nRF52 when the tag has moved beyond a certain threshold.
This could allow for more immediate "target is moving" notifications.

Alternatively, sampling the accelerometer as per usual but advertise only a simple "absolute value" of the acceleration vector. This will give an idea of how much the tag is moving but not which direction(s) it is moving in.

### Temperature (built in)

Every nRF52 chip has a built in temperature sensor.
I'm not really sure what value there is in enabling / transmitting this temperature data; _at best_ it's a crude approximation for the temperature in the immediate vicinity of the chip.

## Home Assistant "timeout" for unavailable

Basically how infrequently can I get away with advertising before HA reports the device as "unavailable"?

There are tradeoffs between a super responsive user experience and battery life.
Traditionally, "user experience" means how fast does the device show up on a phone when scanning for a new BT device but in this case, it's how fast does HA show the device as "home" or "not home".

Broadcasting 10 times a second would mean that the BTHome device would show up in HA well within 1 second of being in range... at the expense of battery life.

All the way at the other end of the spectrum, if the device only broadcasts once every 3 minutes, there would be (up to) a 3 minute delay between the device being in/out of range and HA showing it as "home"/"away".

Is 180 seconds too long? That depends on the use case, I guess.
If you want to have an automation that guarantees all doors are locked as soon as nobody is home, you might want a faster response time.

Initial versions of firmware will have this hard-coded but future versions could have this as a configurable option.

In any case, what is the absolute maximum time that HA will wait before marking a device as "unavailable"?
**Turns out, the answer is 300 seconds / 5 minutes.**

### Testing time to unavailable in HA

To do this test, I used the `LYWSD03MMC` and enabled the `packet_id` sensor in home assistant.

I was seeing the packet ID change every 5-20 seconds.
Usually around 5 seconds but usually under 10 and sometimes as high as 20.

I removed the battery 72 ID and recorded wall-clock time `0910`.
Almost exactly as clock struck `0915`, the device was marked as "unavailable" in HA.

The BTHome docs don't explicitly say how often data needs to be sent so I'm assuming that it's up to the implementer.
As far as I can tell, the 5 min timeout is set [here](https://github.com/home-assistant/core/blob/189c07d502fe67e62d031dd6c4088ef259e2e351/homeassistant/components/bluetooth/const.py#L25) in HA:

```python3
UNAVAILABLE_TRACK_SECONDS: Final = 60 * 5
```

At this time, it does not appear that this is configurable in HA.
I wrote up findings / thoughts [here](https://github.com/Bluetooth-Devices/bthome-ble/issues/121)
