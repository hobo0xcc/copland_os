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

    pub fn _clint_start();
    pub fn _clint_end();
    pub fn _plic_start();
    pub fn _plic_end();
    pub fn _uart0_start();
    pub fn _uart0_end();
    pub fn _virtio_start();
    pub fn _virtio_end();
}

pub const SIFIVE_TEST: usize = 0x100000;
