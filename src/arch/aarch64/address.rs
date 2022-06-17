#![allow(non_upper_case_globals)]

extern "C" {
    pub fn _text_start();
    pub fn _text_end();
    pub fn _rodata_start();
    pub fn _rodata_end();
    pub fn _data_start();
    pub fn _data_end();
    pub fn _bss_start();
    pub fn _bss_end();
    pub fn _stack_start();
    pub fn _stack_end();
    pub fn _heap_start();
    pub fn _heap_end();
}
