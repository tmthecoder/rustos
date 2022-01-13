use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    PhysAddr,
    structures::paging::{PageTable, Page, PhysFrame, Mapper, Size4KiB, FrameAllocator},
    VirtAddr
};
use x86_64::structures::paging::OffsetPageTable;

/// A FrameAllocator that returns usable frames from the bootloader's memory map
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// A convenience method to create a FrameAllocator from the passed memory map
    ///
    /// Unsafe as the caller must guarantee that the passed map is valid
    /// To be valid, all frames marked 'USABLE' must truly be usable
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        // Get the usable regions from the memory map
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // Get a list of address ranges for each region
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // Change ranges to a list of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // Create a 'PhysFrame' type from each frame start address
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // Get the next needed frame
        let frame = self.usable_frames().nth(self.next);
        // Incrememnt the next counter
        self.next += 1;
        frame
    }
}

/// A FrameAllocator that always returns 'None'
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
       None
    }
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

// Creates an example mapping for the given page to frane '0xb8000'
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut dyn FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // FIXME: this isn't safe, just an example
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
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