ENTRY(_start)
MEMORY
{
    ROM (rx) : ORIGIN = 0, LENGTH = 0x1000
    RAM (rwx) : ORIGIN = 0x1000, LENGTH = 0x1000
}
