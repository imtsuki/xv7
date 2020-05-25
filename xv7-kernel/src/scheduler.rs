use crate::cpu::my_cpu;
use crate::paging::VirtAddr;
use crate::process::initcode;
use crate::process::Process;

pub fn scheduler() -> ! {
    let cpu = my_cpu();
    let mut p = Process::new();
    p.set_context_switch_return_address(VirtAddr::new(initcode as *const u8 as u64));

    cpu.current_process = Some(p);

    unsafe {
        cpu.switch_to_process();
    }

    crate::arch::idle();
}
