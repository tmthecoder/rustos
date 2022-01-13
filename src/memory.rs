use x86_64::{PhysAddr, structures::paging::PageTable, VirtAddr};
use x86_64::structures::paging::OffsetPageTable;

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

// A function to retrieve a mutable reference to the active level 4 page table
// Unsafe as it requires a caller guarantee that the physical memory is mapped to the given parameter.
// Also the function can only be called once to avoid aliasing '&mut' references
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    // Get the frame of the level 4 table from the CR3 register
    let (level_4_table_frame, _) = Cr3::read();

    // Get its physcial start address
    let phys = level_4_table_frame.start_address();
    // Add it to the given memory offset
    let virt = physical_memory_offset + phys.as_u64();
    // Get a reference to the page table
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}