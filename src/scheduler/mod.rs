use std::thread;

use crate::vm::{VMEvent, VM};

const MAX_PID: u32 = 50000;

pub struct Scheduler {
    max_pid: u32,
    next_pid: u32,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            next_pid: 0,
            max_pid: MAX_PID,
        }
    }

    pub fn get_thread(&self, mut vm: VM) -> thread::JoinHandle<Vec<VMEvent>> {
        thread::spawn(move || vm.run())
    }
}
