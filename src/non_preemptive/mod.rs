//! Non-preemptive scheduler
//! Basic non-preemptive scheduler to control task execution upon cycle completion
//! and external events which could fit on basic applications

#[cfg(not(feature = "core"))]
compile_error!(
    "Core architecture feature not selected, select one of the following:
        arm-cm
"
);

mod port;
pub mod resources;

use core::str;
use heapless::Vec;
use panic_halt as _;
use port::{critical_section, log, SysTick};

type InitRunnable = fn();
type ProcessRunnable = fn(u32);
type IdleRunnable = fn();
type TaskName = &'static str;
type TaskList<const N: usize> = Vec<Task, N>;
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
    pub const fn new(
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
}

pub struct Scheduler<const TASK_COUNT: usize, const CORE_FREQ: u32> {
    idle_runnable: Option<IdleRunnable>,
    task_list: TaskList<TASK_COUNT>,
}

impl<const TASK_COUNT: usize, const CORE_FREQ: u32> Scheduler<TASK_COUNT, CORE_FREQ> {
    pub const fn new() -> Scheduler<TASK_COUNT, CORE_FREQ> {
        Scheduler {
            idle_runnable: None,
            task_list: TaskList::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.task_list.push(task).unwrap();
    }

    pub fn launch(&mut self) {
        let systick = SysTick::bind_with_core_and_take(CORE_FREQ).unwrap();
        systick.launch();

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
                    systick.get() + execution_cycle + task.execution_offset.unwrap_or(0);
            }
        }

        // Main endless super loop
        loop {
            let mut task_execution = false;
            for task in self.task_list.iter_mut() {
                let mut cyclic_execution = false;
                if let Some(process_runnable) = task.process_runnable {
                    // Update cycle monitor with new absolut time
                    if task.execution_cycle.is_some() && systick.get() >= task.tcb.cycle_monitor {
                        task.tcb.cycle_monitor = systick.get() + task.execution_cycle.unwrap();
                        cyclic_execution = true;
                    }
                    // Execute process runnable if any event set
                    if task.tcb.event_monitor != 0 {
                        let mut event_mask = 0;
                        critical_section(|_| {
                            event_mask = task.tcb.event_monitor;
                            task.tcb.event_monitor = 0;
                        });
                        process_runnable(event_mask);
                        task_execution = true;
                    }
                    // Execute process runnable if cycle period elapsed
                    if cyclic_execution {
                        process_runnable(0);
                        task_execution = true;
                    }
                }
            }
            // Execute idle runnable if registered and there was no execution
            if let Some(idle_runnable) = self.idle_runnable {
                if !task_execution {
                    idle_runnable();
                }
            }
        }
    }

    #[inline]
    pub fn register_idle_runnable(&mut self, idle: fn()) {
        self.idle_runnable = Some(idle);
    }

    #[inline]
    pub fn set_task_event(&mut self, name: &str, event: u32) {
        if let Some(task) = self.task_list.iter_mut().find(|task| task.name == name) {
            critical_section(|_| task.tcb.event_monitor |= event);
        }
    }

    #[inline]
    pub fn clear_task_event(&mut self, name: &str, event: u32) {
        if let Some(task) = self.task_list.iter_mut().find(|task| task.name == name) {
            critical_section(|_| task.tcb.event_monitor &= !event);
        }
    }

    #[inline]
    pub fn get_task_event(&mut self, name: &str) -> Option<u32> {
        if let Some(task) = self.task_list.iter_mut().find(|task| task.name == name) {
            critical_section(|_| Some(task.tcb.event_monitor))
        } else {
            None
        }
    }
}
