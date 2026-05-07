MEMORY
{
    FLASH : ORIGIN = 0x00000000, LENGTH = 1M
    /*  technically we should have 256k of RAM, but it would cause a fault on the device.
        After digging into the official MCUExpresso examples, the linker script defines the 
        RAM region as starting from 0x2000_0000 with a length of 0x3_C000 (240K) */
    RAM   : ORIGIN = 0x20003000, LENGTH = 200K
}
