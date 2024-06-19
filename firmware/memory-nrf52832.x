MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes
  nrf52832 on the HolyIoT 21011 tag has the larger variant: 
  512 KB Flash, 64 KB RAM

  The latest soft device: s112_nrf52_7.3.0 needs
    
    - 100.0 kB (0x19000 bytes) flash.

        TODO: may want to artificially limit flash use to preserve space for OTA?

  RAM depends on how many connections are used and the like.
  This value is tuned / computed for me and emitted to log at run time if too little/much.  
  
  64K RAM total -> 64*1024 -> 65536 => 0x10000
  */
  SOFTDEVICE : ORIGIN = 0x00000000, LENGTH = 100K
  
  FLASH : ORIGIN = 0x00000000 + 100K, LENGTH = 512K - 100K
  
  RAM : ORIGIN = 0x200011b8, LENGTH = 0x10000 - 0x11b8

}


SECTIONS {
  .softdevice :
    {
        KEEP(*(.softdevice .softdevice.*));
    } > SOFTDEVICE
}
