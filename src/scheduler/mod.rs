use std::thread;

use crate::{
    util::display,
    vm::{VMEvent, VM},
};

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
        thread::spawn(move || {
            let events = vm.run();
            display::writeout("VM Events");
            display::writeout("--------------------------");
            for event in &events {
                println!("{:#?}", event);
            }
            events
        })
    }

    pub fn get_next_pid(&self) -> u32 {
        self.next_pid
    }
    pub fn get_max_pid(&self) -> u32 {
        self.max_pid
    }

    fn _next_pid(&mut self) -> u32 {
        let result = self.next_pid;
        self.next_pid += 1;
        result
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use crate::scheduler::Scheduler;

    #[test]
    fn test_make_scheduler() {
        let s = Scheduler::new();
        assert_eq!(s.next_pid, 0);
    }
}
