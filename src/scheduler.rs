//! Scheduler
//! Basic scheduler to control task execution upon cycle completion and events

use core::str;
use heapless::Vec;
use panic_halt as _;

type Name = &'static str;
type Init = fn();
type Process = fn();

#[derive(Debug)]
struct TaskCtrlBlock {
    cycle_monitor: u32,
    event_monitor: u32,
}

#[derive(Debug)]
pub struct Task {
    name: Name,
    init: Option<Init>,
    process: Option<Process>,
    execution_cycle: u32,
    execution_offset: u32,
    tcb: TaskCtrlBlock,
}

impl Task {
    pub fn new(
        name: Name,
        init: Option<Init>,
        process: Option<Process>,
        execution_cycle: u32,
        execution_offset: u32,
    ) -> Task {
        Task {
            name,
            init,
            process,
            execution_cycle,
            execution_offset,
            tcb: TaskCtrlBlock {
                cycle_monitor: 0,
                event_monitor: 0,
            },
        }
    }
}

pub struct Scheduler<const N: usize, F: FnMut() -> u32> {
    timer: F,
    idle_task: Option<fn()>,
    task_list: Vec<Task, N>,
}

impl<const N: usize, F: FnMut() -> u32> Scheduler<N, F> {
    pub fn new(timer: F) -> Scheduler<N, F> {
        Scheduler {
            timer,
            idle_task: None,
            task_list: Vec::<Task, N>::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.task_list.push(task).unwrap();
    }

    pub fn launch(&mut self) {
        for task in self.task_list.iter_mut() {
            // Execute init if any
            if let Some(init) = task.init {
                init();
            }

            // Update execution_cycle monitor if any process function
            if task.process.is_some() {
                task.tcb.cycle_monitor =
                    (self.timer)() + task.execution_cycle + task.execution_offset;
            }
        }

        loop {
            for task in self.task_list.iter_mut() {
                if let Some(process) = task.process {
                    // Check for alarms
                    if (self.timer)() >= task.tcb.cycle_monitor {
                        process();
                        // Update cycle monitor with new absolut time
                        task.tcb.cycle_monitor = (self.timer)() + task.execution_cycle;
                    }
                    // TODO: Handle events
                }
            }

            // Execute idle task if registered
            self.idle_task.unwrap_or(|| {});
        }
    }

    pub fn register_idle_task(&mut self, idle: fn()) {
        self.idle_task = Some(idle);
    }

    pub fn _set_event(&mut self, name: &str, event: u32) {
        if let Some(task) = self.task_list.iter_mut().find(|x| x.name == name) {
            task.tcb.event_monitor |= event;
        }
    }

    pub fn _get_event(&mut self, name: &str) -> Option<u32> {
        if let Some(task) = self.task_list.iter_mut().find(|x| x.name == name) {
            Some(task.tcb.event_monitor)
        } else {
            None
        }
    }
}
