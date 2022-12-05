//! Non-preemptive scheduler
//! Basic non-preemptive scheduler to control task execution upon cycle completion
//! and external events which could fit on basic applications

use core::str;
use cortex_m::interrupt::free as critical_section;
use heapless::Vec;
use panic_halt as _;
use rtt_target::rprintln as log;

type InitRunnable = fn();
type ProcessRunnable = fn(u32);
type IdleRunnable = fn();
type TimeMonitor = fn() -> u32;
type TaskName = &'static str;
type TaskList<'a, const N: usize> = Vec<&'a mut Task, N>;
pub type EventMask = u32;

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
    execution_cycle: Option<u32>,
    execution_offset: Option<u32>,
    tcb: TaskCtrlBlock,
}

impl Task {
    pub fn new(
        name: TaskName,
        init_runnable: Option<InitRunnable>,
        process_runnable: Option<ProcessRunnable>,
        execution_cycle: Option<u32>,
        execution_offset: Option<u32>,
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

    #[inline]
    pub fn set_event(&mut self, event: EventMask) {
        critical_section(|_| {
            self.tcb.event_monitor = event;
        })
    }
}

pub struct Scheduler<'a, const N: usize> {
    pub time_monitor: TimeMonitor,
    pub idle_runnable: Option<IdleRunnable>,
    pub task_list: TaskList<'a, N>,
}

impl<'a, const N: usize> Scheduler<'a, N> {
    pub fn new(time_monitor: TimeMonitor) -> Scheduler<'a, N> {
        Scheduler {
            time_monitor,
            idle_runnable: None,
            task_list: TaskList::new(),
        }
    }

    pub fn add_task(&mut self, task: &'a mut Task) {
        self.task_list.push(task).unwrap();
    }

    pub fn launch(&mut self) {
        for task in self.task_list.iter_mut() {
            log!("Launching task {}", task.name);

            // Execute init_runnable if any
            if let Some(init_runnable) = task.init_runnable {
                init_runnable();
            }

            // Update cycle monitor if any process_runnable function and exeuction_cycle configured
            if let (Some(_), Some(execution_cycle)) = (task.process_runnable, task.execution_cycle)
            {
                task.tcb.cycle_monitor =
                    (self.time_monitor)() + execution_cycle + task.execution_offset.unwrap_or(0);
            }
        }

        // Main endless super loop
        loop {
            for task in self.task_list.iter_mut() {
                if let Some(process_runnable) = task.process_runnable {
                    // Execute process runnable if any event set
                    if task.tcb.event_monitor != 0 {
                        let mut event_mask = 0;
                        critical_section(|_| {
                            event_mask = task.tcb.event_monitor;
                            task.tcb.event_monitor = 0;
                        });
                        process_runnable(event_mask);
                    }
                    // Execute process runnable if cycle period elapsed
                    if task.execution_cycle.is_some()
                        && (self.time_monitor)() >= task.tcb.cycle_monitor
                    {
                        process_runnable(task.tcb.event_monitor);
                        // Update cycle monitor with new absolut time
                        task.tcb.cycle_monitor =
                            (self.time_monitor)() + task.execution_cycle.unwrap();
                    }
                }
            }

            // Execute idle runnable if registered
            if let Some(idle_runnable) = self.idle_runnable {
                idle_runnable();
            }
        }
    }

    #[inline]
    pub fn register_idle_runnable(&mut self, idle: fn()) {
        self.idle_runnable = Some(idle);
    }

    #[inline]
    pub fn set_event(&mut self, name: &str, event: u32) {
        if let Some(task) = self.task_list.iter_mut().find(|task| task.name == name) {
            critical_section(|_| task.tcb.event_monitor |= event);
        }
    }

    #[inline]
    pub fn clear_event(&mut self, name: &str, event: u32) {
        if let Some(task) = self.task_list.iter_mut().find(|task| task.name == name) {
            critical_section(|_| task.tcb.event_monitor &= !event);
        }
    }

    #[inline]
    pub fn get_event(&mut self, name: &str) -> Option<u32> {
        if let Some(task) = self.task_list.iter_mut().find(|task| task.name == name) {
            critical_section(|_| Some(task.tcb.event_monitor))
        } else {
            None
        }
    }
}