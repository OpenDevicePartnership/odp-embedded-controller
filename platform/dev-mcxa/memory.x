MEMORY
{
    FLASH : ORIGIN = 0x00000000, LENGTH = 1M
    /*  technically we should have 256KiB of RAM, but it would cause a fault on the device.
        After digging into the official MCUExpresso examples, the linker script defines the 
        RAM region as starting from 0x2000_0000 with a length of 0x3_C000 (240K)
        Based on our own testing, the first 0x3000 bytes in memory are wiped out when resetting
        the chip. For testing purposes where we may want to run from RAM, this will corrupt that
        region so start at 0x2000_3000. Length is 240KiB - 12KiB = 228KiB
     */
    RAM   : ORIGIN = 0x20003000, LENGTH = 228K
}
