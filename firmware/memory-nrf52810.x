MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes
    nrf52810 has 192K of flash and 24K of RAM
  */
  
  /*  Technically this is the MBR and the soft device; they start at the beginning of flash.
        Release notes for each version of the soft device indicate how much space it takes up
        Values below assume s112_nrf52_7.3.0
  */
  
  SOFTDEVICE : ORIGIN = 0x00000000, LENGTH = 100K
  FLASH : ORIGIN = 0x00019000, LENGTH = 192K - 100K
  
  /* 
    24K RAM total -> 24*1024 -> 24576 => 0x6000
    
    Note that the exact address changes based on Config of soft-device.

    After quite some testing / tuning, the lowest possible ram use that I could get
    with the s112@7.0 is:

      - advertise-only program: 0x11b8 bytes
          0x6000 - 0x11b8 = 20040 bytes left over
      
      Even with the absolute bare minimum soft-device config, the 810 just doesn't
        have enough RAM to run the soft-device and and of the more sophisticated programs
        at the same time.
      For this reason, pivoting to the 832 as it's only slightly more expensive but has 2.7 times
        the RAM!
   */
  RAM : ORIGIN = 0x200011b8, LENGTH = 24K - 0x11b8

}

/*
  This allows us to ship the nordic softdevice binary as part of the
  compiled binary.
  In testing, linker does not really complain if the .softdevice section is
    not actually found / has no variables stuffed into it.
  This way, we can keep the memory file consistent and use a feature gate to
    change if the softdevice is included or not.
*/
SECTIONS {
  .softdevice :
    {
        KEEP(*(.softdevice .softdevice.*));
    } > SOFTDEVICE
}
