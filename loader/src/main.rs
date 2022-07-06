#![no_std]
#![no_main]
#![feature(abi_efiapi, vec_into_raw_parts)]

use goblin::elf;
use uefi::prelude::*;
use uefi::proto::console::gop::*;
use uefi::proto::media::file::{Directory, File, FileAttribute, FileInfo, FileMode};
use uefi::table::boot::*;
use uefi::Status;

use alloc::format;
use alloc::vec;
use alloc::vec::*;
use core::mem;
use uefi::table::cfg::ACPI_GUID;

use common::uefi::*;

extern crate alloc;

#[no_mangle]
pub static mut KERNEL_STACK_MAIN: [u8; 4096] = [0; 4096];

macro_rules! fwrite {
    ($file:expr, $format:tt $( $rest:tt )*) => {
        $file.write(format!($format $( $rest )*).as_bytes()).unwrap()
    };
}

macro_rules! fwriteln {
    ($file:expr, $format:tt $( $rest:tt )*) => {
        fwrite!($file, concat!($format, "\n") $( $rest )*)
    };
}

fn round_up(n: usize, round: usize) -> usize {
    ((n + (round - 1)) / round) * round
}

fn open_root_dir(image: Handle, bs: &BootServices) -> Directory {
    let simple_fs = bs.get_image_file_system(image).unwrap();
    unsafe { simple_fs.interface.get().as_mut().unwrap() }
        .open_volume()
        .unwrap()
}

fn is_available_after_exit_boot_services(ty: MemoryType) -> bool {
    matches!(
        ty,
        MemoryType::CONVENTIONAL | MemoryType::BOOT_SERVICES_CODE | MemoryType::BOOT_SERVICES_DATA
    )
}

fn get_framebuffer(bs: &BootServices) -> common::uefi::FrameBuffer {
    let gop = unsafe {
        bs.locate_protocol::<GraphicsOutput>()
            .unwrap()
            .get()
            .as_mut()
            .unwrap()
    };
    let mode_info = gop.current_mode_info();
    common::uefi::FrameBuffer {
        ptr: gop.frame_buffer().as_mut_ptr() as *mut u32,
        width: mode_info.resolution().0,
        height: mode_info.resolution().1,
        stride: mode_info.stride(),
        format: match mode_info.pixel_format() {
            PixelFormat::Rgb => FrameBufferFormat::RGB,
            PixelFormat::Bgr => FrameBufferFormat::BGR,
            format => panic!("unknown format: {:?}", format),
        },
    }
}

fn get_rsdp(st: &SystemTable<Boot>) -> usize {
    st.config_table()
        .iter()
        .find(|config| config.guid == ACPI_GUID)
        .map(|config| config.address)
        .expect("rsdp") as usize
}

#[entry]
fn efi_main(image: Handle, mut st: SystemTable<Boot>) -> Status {
    use core::fmt::Write;
    uefi_services::init(&mut st).unwrap();
    st.stdout().reset(false).unwrap();
    writeln!(st.stdout(), "Hello, world!").unwrap();

    let mut root = open_root_dir(image, st.boot_services());
    let mut dump_to = root
        .open(
            cstr16!("memmap"),
            FileMode::CreateReadWrite,
            FileAttribute::empty(),
        )
        .expect("create")
        .into_regular_file()
        .unwrap();
    let size =
        st.boot_services().memory_map_size().map_size + 16 * mem::size_of::<MemoryDescriptor>();
    let mut buf = vec![0; size];
    let (_, descs) = st.boot_services().memory_map(&mut buf).unwrap();
    for desc in descs {
        fwriteln!(
            dump_to,
            "ty: {:?}, phys_start: {:#x}, virt_start: {:#x}, page_count: {}",
            desc.ty,
            desc.phys_start,
            desc.virt_start,
            desc.page_count
        );
    }
    dump_to.close();

    let mut framebuffer = get_framebuffer(st.boot_services());
    let rsdp_addr = get_rsdp(&st);

    let mut kernel_file = root
        .open(
            cstr16!("kernel.elf"),
            FileMode::Read,
            FileAttribute::empty(),
        )
        .expect("kernel.elf")
        .into_regular_file()
        .unwrap();
    let kernel_file_info = kernel_file.get_boxed_info::<FileInfo>().unwrap();
    let mut buf: Vec<u8> = vec![0; kernel_file_info.file_size() as usize];
    kernel_file.read(&mut buf).unwrap();
    let elf_exec =
        elf::Elf::parse(&buf).expect("cannot parse as an elf file: /EFI/BOOT/kernel.elf");
    let mut start = usize::MAX;
    let mut end = 0;
    for ph in elf_exec.program_headers.iter() {
        if ph.p_type != elf::program_header::PT_LOAD {
            continue;
        }
        start = start.min(ph.p_vaddr as usize);
        end = end.max((ph.p_vaddr + ph.p_memsz) as usize);
    }

    let size_bytes = end - start;
    st.boot_services()
        .allocate_pages(
            AllocateType::Address(start),
            MemoryType::LOADER_DATA,
            round_up(size_bytes, PAGE_SIZE) / PAGE_SIZE,
        )
        .unwrap();

    for ph in elf_exec.program_headers.iter() {
        if ph.p_type != elf::program_header::PT_LOAD {
            continue;
        }
        let offset = ph.p_offset as usize;
        let file_size = ph.p_filesz as usize;
        let mem_size = ph.p_memsz as usize;
        let data = unsafe { core::slice::from_raw_parts_mut(ph.p_vaddr as *mut u8, mem_size) };
        data[..file_size].copy_from_slice(&buf[offset..(offset + file_size)]);
        data[file_size..].fill(0);
    }

    let mmap_size =
        st.boot_services().memory_map_size().map_size + 8 * mem::size_of::<MemoryDescriptor>();
    let mut mmap_buf: Vec<u8> = vec![0; mmap_size];
    let mut descriptors = Vec::with_capacity(mmap_size);
    let (_st, raw_descriptors) = st
        .exit_boot_services(image, &mut mmap_buf)
        .expect("exit boot services");

    for desc in raw_descriptors {
        if is_available_after_exit_boot_services(desc.ty) {
            descriptors.push(MemoryRegion {
                start: desc.phys_start as usize,
                end: (desc.phys_start + desc.page_count * PAGE_SIZE as u64) as usize,
            });
        }
    }

    let mut memory_map = {
        let (ptr, len, _) = descriptors.into_raw_parts();
        MemoryMap {
            descriptors: ptr as *const MemoryRegion,
            len,
        }
    };

    let entry_addr = elf_exec.header.e_entry as usize;
    let kernel_entry = unsafe {
        core::mem::transmute::<
            *mut u8,
            extern "sysv64" fn(&mut MemoryMap, &mut common::uefi::FrameBuffer, usize) -> !,
        >(entry_addr as *mut u8)
    };
    kernel_entry(&mut memory_map, &mut framebuffer, rsdp_addr);
}
