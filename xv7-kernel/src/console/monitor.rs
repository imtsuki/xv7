use super::Console;

pub struct MonitorConsole;

impl Console for MonitorConsole {
    fn write(&mut self, _buf: &[u8]) {
        todo!();
    }

    fn read(&mut self, _buf: &mut [u8]) -> usize {
        todo!();
    }
}
