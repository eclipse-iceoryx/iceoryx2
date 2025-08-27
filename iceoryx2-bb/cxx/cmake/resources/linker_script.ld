SECTIONS
{
    .debug_gdb_scripts : {
        *start_byte.o(.data)
        *filename_byte.o(.data)
        *lf_byte.o(.data)
        *pretty_printer_script.o(.data)
        *end_byte.o(.data)
    }
}

