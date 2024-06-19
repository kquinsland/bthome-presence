<!-- markdownlint-disable-file MD041 -->
Quick and dirty notes from testing stock firmware.

`test01` was initial power up, connect with iPad and then dial down to lowest possible advert power with highest possible interval.
The app does not explain power levels; they're just on a scale from 1-7. Max advert interval is 4s.

`test02` was re-connecting with iPad and dialing up transmit to max, keeping the power interval.
Transmits were around 6mA costing me a total of 72.54 uA/10s.

Interestingly enough, the app would show battery / transmit interval as properties.

I didn't investigate if this is in the advert payload or in a follow up payload.

Device name was `R23030253`.

JK. I forgot to hit "save" on the power level.

`test03` is with the power all the way up to level 9, still 4000ms interval.

That costs 91.42 uA/10s.

The advertisement data as seen by nrfConnect before / after the change doesn't change much, but it does [change](https://gchq.github.io/CyberChef/#recipe=Diff('%5C%5Cn%5C%5Cn','Character',true,true,false,false)&input=MHgwMjAxMDYxQUZGNEMwMDAyMTVGREE1MDY5M0E0RTI0RkIxQUZDRkM2RUIwNzY0NzgyNTAwMDEwMDAyRDgwQTA5NTIzMjMzMzAzMzMwMzIzNTMzMTExNjAzMTg1MjAxMjMwMzAwRkQwMDAxMDAwMjA3MDNFODY0CgoweDAyMDEwNjFBRkY0QzAwMDIxNUZEQTUwNjkzQTRFMjRGQjFBRkNGQzZFQjA3NjQ3ODI1MDAwMTAwMDJEODBBMDk1MjMyMzMzMDMzMzAzMjM1MzMxMTE2MDMxODUyMDEyMzAzMDBGRDAwMDEwMDAyMDkwRkEwNjQ).

Before:

```text
0x0201061AFF4C000215FDA50693A4E24FB1AFCFC6EB0764782500010002D80A09523233303330323533111603185201230300FD000100020703E864
```

After:

```text
0x0201061AFF4C000215FDA50693A4E24FB1AFCFC6EB0764782500010002D80A09523233303330323533111603185201230300FD00010002090FA064
```

```text
B: 703E864
A: 90FA064
```

There is a difference so it absolutely could be that this device with stock firmware could be used with a custom lambda on ESP to do battery tracking.

### Apps

OEM recommends these apps:

iOS: [RLBeacon Tool](https://apps.apple.com/us/app/radioland/id1450730006)
Android: I can't find a working link to the app they suggest. I did find this, but it's not an APK: http://www.radioland-china.com/Android.html
