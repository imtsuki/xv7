use core::cell::SyncUnsafeCell;
use core::mem::{offset_of, size_of};

use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;

const RING3_RPL: u16 = 3;

/// GDT segment selectors (layout order matches `IA32_STAR` MSR requirements for `syscall`/`sysret`).
pub const KERNEL_CODE_SELECTOR: u16 = 1 << 3;
pub const KERNEL_DATA_SELECTOR: u16 = 2 << 3;
pub const USER_DATA_SELECTOR: u16 = (3 << 3) | RING3_RPL;
pub const USER_CODE_SELECTOR: u16 = (4 << 3) | RING3_RPL;
pub const TSS_SELECTOR: u16 = 5 << 3;

// 64-bit segment descriptor bit flags (Intel SDM Vol. 3A, 3.4.5).
const SEG_WRITABLE: u64 = 1 << 41;
const SEG_EXECUTABLE: u64 = 1 << 43;
const SEG_CODE_OR_DATA: u64 = 1 << 44;
const SEG_DPL_RING3: u64 = 3 << 45;
const SEG_PRESENT: u64 = 1 << 47;
const SEG_LONG_MODE: u64 = 1 << 53;

const KERNEL_CODE_DESC: u64 =
    SEG_CODE_OR_DATA | SEG_PRESENT | SEG_EXECUTABLE | SEG_WRITABLE | SEG_LONG_MODE;
const KERNEL_DATA_DESC: u64 = SEG_CODE_OR_DATA | SEG_PRESENT | SEG_WRITABLE;
const USER_DATA_DESC: u64 = SEG_CODE_OR_DATA | SEG_PRESENT | SEG_WRITABLE | SEG_DPL_RING3;
const USER_CODE_DESC: u64 =
    SEG_CODE_OR_DATA | SEG_PRESENT | SEG_EXECUTABLE | SEG_WRITABLE | SEG_LONG_MODE | SEG_DPL_RING3;

const IST_STACK_SIZE: usize = 4096;

#[repr(C, align(16))]
struct IstStack([u8; IST_STACK_SIZE]);

static DOUBLE_FAULT_STACK: SyncUnsafeCell<IstStack> =
    SyncUnsafeCell::new(IstStack([0; IST_STACK_SIZE]));
static PAGE_FAULT_STACK: SyncUnsafeCell<IstStack> =
    SyncUnsafeCell::new(IstStack([0; IST_STACK_SIZE]));

/// Architectural 64-bit Task State Segment (`104` bytes).
#[repr(C, packed(4))]
struct TaskStateSegment {
    reserved_1: u32,
    rsp: [u64; 3],
    reserved_2: u64,
    ist: [u64; 7],
    reserved_3: u64,
    reserved_4: u16,
    iomap_base: u16,
}

const _: () = {
    assert!(offset_of!(TaskStateSegment, rsp) == 0x04);
    assert!(offset_of!(TaskStateSegment, ist) == 0x24);
    assert!(offset_of!(TaskStateSegment, iomap_base) == 0x66);
    assert!(size_of::<TaskStateSegment>() == 0x68);
};

impl TaskStateSegment {
    const fn new() -> Self {
        Self {
            reserved_1: 0,
            rsp: [0; 3],
            reserved_2: 0,
            ist: [0; 7],
            reserved_3: 0,
            reserved_4: 0,
            iomap_base: size_of::<Self>() as u16,
        }
    }
}

/// GDT: `[null, kernel_code, kernel_data, user_data, user_code, tss_low, tss_high]`.
static GDT: SyncUnsafeCell<[u64; 7]> = SyncUnsafeCell::new([
    0,
    KERNEL_CODE_DESC,
    KERNEL_DATA_DESC,
    USER_DATA_DESC,
    USER_CODE_DESC,
    0,
    0,
]);

static TSS: SyncUnsafeCell<TaskStateSegment> = SyncUnsafeCell::new(TaskStateSegment::new());

#[repr(C, packed(2))]
struct Gdtr {
    limit: u16,
    base: u64,
}

/// Encodes a 16-byte 64-bit Available TSS descriptor (`type = 0x9`).
fn encode_tss_descriptor(base: u64) -> (u64, u64) {
    let limit = (size_of::<TaskStateSegment>() - 1) as u64;
    let low = (limit & 0xFFFF)
        | ((base & 0xFF_FFFF) << 16)
        | (0x9 << 40) // Type = 64-bit Available TSS (0b1001), S = 0
        | SEG_PRESENT
        | (((limit >> 16) & 0xF) << 48)
        | (((base >> 24) & 0xFF) << 56);
    let high = base >> 32;
    (low, high)
}

pub fn init() {
    let tss_ptr = TSS.get();
    let gdt_ptr = GDT.get();

    unsafe {
        (*tss_ptr).ist[DOUBLE_FAULT_IST_INDEX as usize] =
            DOUBLE_FAULT_STACK.get() as u64 + IST_STACK_SIZE as u64;
        (*tss_ptr).ist[PAGE_FAULT_IST_INDEX as usize] =
            PAGE_FAULT_STACK.get() as u64 + IST_STACK_SIZE as u64;

        let (tss_low, tss_high) = encode_tss_descriptor(tss_ptr as u64);
        (*gdt_ptr)[5] = tss_low;
        (*gdt_ptr)[6] = tss_high;

        let gdtr = Gdtr {
            limit: (size_of::<[u64; 7]>() - 1) as u16,
            base: gdt_ptr as u64,
        };

        core::arch::asm!(
            "lgdt [{gdtr}]",
            "push {kcs}",
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",
            "2:",
            "mov {tmp:x}, {kds}",
            "mov ss, {tmp:x}",
            "mov ds, {tmp:x}",
            "mov es, {tmp:x}",
            "mov gs, {tmp:x}",
            "mov {tmp:x}, {tss}",
            "ltr {tmp:x}",
            gdtr = in(reg) &gdtr,
            tmp = out(reg) _,
            kcs = const KERNEL_CODE_SELECTOR as usize,
            kds = const KERNEL_DATA_SELECTOR,
            tss = const TSS_SELECTOR,
            options(preserves_flags),
        );
    }
}

/// Updates `TSS.rsp[0]` (`RSP0`) for hardware Ring 3 -> Ring 0 transitions.
pub fn set_tss_rsp0(kstack_top: VirtAddr) {
    unsafe {
        (*TSS.get()).rsp[0] = kstack_top.as_u64();
    }
}
