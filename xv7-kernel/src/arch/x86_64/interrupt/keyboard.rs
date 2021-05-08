use super::controller::LOCAL_APIC;
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
        Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
    );
}

pub extern "x86-interrupt" fn handler(_stack_frame: InterruptStackFrame) {
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::<u8>::new(0x60);
    let scancode = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(c) => {
                    print!("{}", c);
                    let mut buf = [0; 1];
                    c.encode_utf8(&mut buf);

                    match crate::device::console::KEYBOARD_BUFFER.push(buf[0]) {
                        Ok(()) => (),
                        Err(_) => println!("key queue full; dropping key"),
                    }
                }
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }
    LOCAL_APIC.lock().end_of_interrupt();
}
