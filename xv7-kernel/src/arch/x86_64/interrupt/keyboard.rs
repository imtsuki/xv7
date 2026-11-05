use super::controller::LOCAL_APIC;
use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, HandleControl, PS2Keyboard, ScancodeSet1, layouts};
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;

lazy_static! {
    static ref KEYBOARD: Mutex<PS2Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(PS2Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore
        ));
}

pub extern "x86-interrupt" fn handler(_stack_frame: InterruptStackFrame) {
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::<u8>::new(0x60);
    let scancode = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            drop(keyboard);
            match key {
                DecodedKey::Unicode(c) => {
                    let mut buf = [0u8; 4];
                    let encoded = c.encode_utf8(&mut buf);
                    for &b in encoded.as_bytes() {
                        crate::device::tty::push_raw_byte(b);
                    }
                }
                DecodedKey::RawKey(_) => {}
            }
        }
    }
    LOCAL_APIC.lock().end_of_interrupt();
}
