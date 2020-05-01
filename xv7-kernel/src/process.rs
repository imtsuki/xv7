pub struct Context;

pub enum ProcessState {
    Spawn,
    Runnable,
    Running,
    Zombie,
}

pub struct Process {
    pub pid: u64,
    pub context: Context,
    pub state: ProcessState,
}
