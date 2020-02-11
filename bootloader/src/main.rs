#![no_std]
#![no_main]
#![feature(asm)]
#![feature(abi_efiapi)]

#[macro_use]
extern crate alloc;
use log::*;

use alloc::boxed::Box;
use uefi::prelude::*;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, RegularFile};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::AllocateType;
use uefi::table::cfg::ACPI2_GUID;
use x86_64::registers::control::{Cr0, Cr0Flags, Cr3, Efer, EferFlags};
use x86_64::structures::paging::{
    FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB, UnusedPhysFrame,
};
use x86_64::{PhysAddr, VirtAddr};
use xmas_elf::ElfFile;

use bootloader::{BootInfo, MemoryMap, MemoryType};

pub mod console;
pub mod page_table;

const KERNEL_PATH: &str = "\\EFI\\kernel\\kernel.elf";
const PHYSICAL_MEMORY_OFFSET: u64 = 0xFFFF800000000000;
type KernelEntry = extern "C" fn(*const BootInfo) -> !;

#[entry]
fn efi_main(handle: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&system_table).expect_success("failed to initialize utilities");
    // 清除UEFI之前打印的东西.
    system_table
        .stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");
    // Test output
    info!("Hello UEFI!");

    let boot_services = system_table.boot_services();

    // RSDP（根系统描述指针）是ACPI编程接口中使用的数据结构。
    // 寻找RSDP
    // 如果您使用的是UEFI，则可以在EFI_SYSTEM_TABLE中的某个位置找到它。因此，无需搜索RAM。
    let acpi2_addr = system_table
        .config_table()
        .iter()
        .find(|entry| entry.guid == ACPI2_GUID)
        .expect("Failed to find RSDP")
        .address;

    info!("ACPI2 RSDP address is : {:?}", acpi2_addr);

    // 获取memory map
    let max_mmap_size = boot_services.memory_map_size();
    let mmap_storage = Box::leak(vec![0; max_mmap_size].into_boxed_slice());

    // 从CR3获取当前页表
    let mut page_table = {
        let p4_table_addr = Cr3::read().0.start_address().as_u64();
        let p4_table = unsafe { &mut *(p4_table_addr as *mut PageTable) };
        unsafe { OffsetPageTable::new(p4_table, VirtAddr::new(0)) }
    };

    // 根页表是只读的
    // 禁用写入保护
    unsafe {
        Cr0::update(|f| f.remove(Cr0Flags::WRITE_PROTECT));
        Efer::update(|f| f.insert(EferFlags::NO_EXECUTE_ENABLE));
    }

    let elf = ElfFile::new(load_kernel(boot_services)).expect("Failed to parse ELF");
    let entry = elf.header.pt2.entry_point() as usize;

    page_table::map_elf(
        &elf,
        &mut page_table,
        &mut UEFIFrameAllocator(boot_services),
    )
    .expect("failed to map ELF");

    // 恢复写入保护
    unsafe {
        Cr0::update(|f| f.insert(Cr0Flags::WRITE_PROTECT));
    }

    // 退出启动服务,启动kernel
    let (_rt, mmap_iter) = system_table
        .exit_boot_services(handle, mmap_storage)
        .expect_success("Failed to exit boot services");

    // construct BootInfo
    let boot_info = BootInfo {
        memory_map: MemoryMap { iter: mmap_iter },
        physical_memory_offset: PHYSICAL_MEMORY_OFFSET,
        acpi2_rsdp_addr: acpi2_addr as u64,
    };

    // 将bootinfo传递给内核,并跳转到内核
    jump_to_entry(&boot_info, entry);
}

fn load_kernel(boot_services: &BootServices) -> &'static mut [u8] {
    info!("Loading file: {}", KERNEL_PATH);
    let file_system = boot_services
        .locate_protocol::<SimpleFileSystem>()
        .expect_success("Failed to get FileSystem");
    let file_system = unsafe { &mut *file_system.get() };
    let mut root = file_system
        .open_volume()
        .expect_success("Failed to open volumes");
    let file_handle = root
        .open(KERNEL_PATH, FileMode::ReadWrite, FileAttribute::empty())
        .expect_success("Failed to open file");
    let mut file_handle = unsafe { RegularFile::new(file_handle) };
    info!("Loading file to memory");
    let mut info_buf = [0u8; 0x100];
    let info = file_handle
        .get_info::<FileInfo>(&mut info_buf)
        .expect_success("Failed to get file info");
    let pages = info.file_size() as usize / 0x1000 + 1;
    let mem_start = boot_services
        .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, pages)
        .expect_success("Failed to allocate pages");
    let buf = unsafe { core::slice::from_raw_parts_mut(mem_start as *mut u8, pages * 0x1000) };
    let len: usize = file_handle.read(buf).expect_success("Failed to read file");
    &mut buf[..len]
}

/// 使用 `BootServices::allocate_pages()` 作为frame分配器
struct UEFIFrameAllocator<'a>(&'a BootServices);

unsafe impl FrameAllocator<Size4KiB> for UEFIFrameAllocator<'_> {
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame<Size4KiB>> {
        let addr = self
            .0
            .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
            .expect_success("failed to allocate frame");
        let frame = PhysFrame::containing_address(PhysAddr::new(addr));
        Some(unsafe { UnusedPhysFrame::new(frame) })
    }
}

/// 根据全局变量'entry'跳转到ELF条目
fn jump_to_entry(boot_info: *const BootInfo, entry: usize) -> ! {
    println!("Jump to Kernel...");

    // 设置 cargo xbuild 时无法使用，但是 cargo xbuild --release则可以正常使用
    let entry_kernel: KernelEntry = unsafe { core::mem::transmute(entry) };
    entry_kernel(boot_info)

    /*    // Debug or Release can use this
    unsafe {
        // TODO: Setup stack pointer safely
        //       Now rsp is pointing to physical mapping area without guard page.
        asm!("add rsp, $0; jmp $1"
            :: "m"(PHYSICAL_MEMORY_OFFSET), "r"(entry), "{rdi}"(boot_info)
            :: "intel");
    }
    unreachable!()*/
}
