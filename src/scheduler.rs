//! Scheduler
//! Basic scheduler to control task execution upon cycle completion and events

use core::str;
use heapless::Vec;
use panic_halt as _;
use rtt_target::rprintln as log;

type InitRunnable = fn();
type ProcessRunnable = fn();
type TimeMonitor = fn() -> u32;
type TaskName = &'static str;
type TaskList<const N: usize> = Vec<Task, N>;

#[derive(Debug)]
struct TaskCtrlBlock {
    cycle_monitor: u32,
    event_monitor: u32,
}

#[derive(Debug)]
pub struct Task {
    name: TaskName,
    init_runnable: Option<InitRunnable>,
    process_runnable: Option<ProcessRunnable>,
    execution_cycle: u32,
    execution_offset: u32,
    tcb: TaskCtrlBlock,
}

impl Task {
    pub fn new(
        name: TaskName,
        init_runnable: Option<InitRunnable>,
        process_runnable: Option<ProcessRunnable>,
        execution_cycle: u32,
        execution_offset: u32,
    ) -> Task {
        Task {
            name,
            init_runnable,
            process_runnable,
            execution_cycle,
            execution_offset,
            tcb: TaskCtrlBlock {
                cycle_monitor: 0,
                event_monitor: 0,
            },
        }
    }
}

pub struct Scheduler<const N: usize> {
    time_monitor: TimeMonitor,
    idle_task: Option<fn()>,
    task_list: TaskList<N>,
}

impl<const N: usize> Scheduler<N> {
    pub fn new(time_monitor: TimeMonitor) -> Scheduler<N> {
        Scheduler {
            time_monitor,
            idle_task: None,
            task_list: Vec::<Task, N>::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.task_list.push(task).unwrap();
    }

    pub fn launch(&mut self) {
        for task in self.task_list.iter_mut() {
            log!("Launching task {}", task.name);

            // Execute init_runnable if any
            if let Some(init_runnable) = task.init_runnable {
                init_runnable();
            }

            // Update cycle monitor if any process_runnable function
            if task.process_runnable.is_some() {
                task.tcb.cycle_monitor =
                    (self.time_monitor)() + task.execution_cycle + task.execution_offset;
            }
        }

        // Main endless super loop
        loop {
            for task in self.task_list.iter_mut() {
                if let Some(process_runnable) = task.process_runnable {
                    // Check for alarms
                    if (self.time_monitor)() >= task.tcb.cycle_monitor {
                        process_runnable();
                        // Update cycle monitor with new absolut time
                        task.tcb.cycle_monitor = (self.time_monitor)() + task.execution_cycle;
                    }
                    // TODO: Check for events
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
        if let Some(task) = self.task_list.iter_mut().find(|task| task.name == name) {
            task.tcb.event_monitor |= event;
        }
    }

    pub fn _get_event(&mut self, name: &str) -> Option<u32> {
        if let Some(task) = self.task_list.iter_mut().find(|task| task.name == name) {
            Some(task.tcb.event_monitor)
        } else {
            None
        }
    }
}
