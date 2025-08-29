SECTIONS
{
    .debug_gdb_scripts : {
        BYTE(0x04)
        *pretty_printer_filename.o(.data)
        BYTE(0x0a)
        *pretty_printer_script.o(.data)
        BYTE(0x00)
    }
}

