//! Non-preemptive scheduler
//! Basic non-preemptive scheduler to control task execution upon cycle completion
//! and external events which could fit on basic applications

#![cfg_attr(not(test), no_std)]

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
#[cfg(debug_assertions)]
use port::log;
use port::{critical_section, SysTick};

pub type InitRunnable = fn();
pub type ProcessRunnable = fn(u32);
pub type IdleRunnable = fn();
pub type TaskName = &'static str;
pub type EventMask = u32;
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

    pub fn has_duplicates_of(&self, other: &Self) -> bool {
        self.name == other.name
            || self.has_same_init_runnable_as(other)
            || self.has_same_process_runnable_as(other)
    }

    fn has_same_init_runnable_as(&self, other: &Self) -> bool {
        if let (Some(init_runnable), Some(other_init_runnable)) =
            (self.init_runnable, other.init_runnable)
        {
            init_runnable == other_init_runnable
        } else {
            false
        }
    }

    fn has_same_process_runnable_as(&self, other: &Self) -> bool {
        if let (Some(process_runnable), Some(other_process_runnable)) =
            (self.process_runnable, other.process_runnable)
        {
            process_runnable == other_process_runnable
        } else {
            false
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
        #[cfg(debug_assertions)]
        log!(
            "Adding task {} to scheduler: \n \
              - init runnable: {:?}\n \
              - process runnable: {:?}\n \
              - execution cycle: {:?}\n \
              - execution offset: {:?}",
            task.name,
            task.init_runnable,
            task.process_runnable,
            task.execution_cycle,
            task.execution_offset
        );
        self.check_if_task_has_duplicates(&task);
        if let Err(task) = self.task_list.push(task) {
            panic!("Task {} cannot be added, task list already full", task.name);
        }
    }

    pub fn launch(&mut self) {
        let systick = SysTick::bind_with_core_and_take(CORE_FREQ).unwrap();
        systick.launch();

        for task in self.task_list.iter_mut() {
            #[cfg(debug_assertions)]
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
                    if let Some(execution_cycle) = task.execution_cycle {
                        if systick.get() >= task.tcb.cycle_monitor {
                            task.tcb.cycle_monitor = systick.get() + execution_cycle;
                            cyclic_execution = true;
                        }
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
        if let Some(task) = self.task_list.iter().find(|task| task.name == name) {
            critical_section(|_| Some(task.tcb.event_monitor))
        } else {
            None
        }
    }

    fn check_if_task_has_duplicates(&self, task: &Task) {
        for added_task in self.task_list.iter() {
            if task.has_duplicates_of(added_task) {
                panic!(
                    "Task {} has either same name or init runnable: {:?} \
                    or process runnable: {:?} than an already added task {}",
                    task.name, task.init_runnable, task.process_runnable, added_task.name
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DUMMY_CORE_FREQ: u32 = 100_000_000;

    #[test]
    fn task_event_handling() {
        const TASK_COUNT: usize = 1;
        const TASK_NAME: &str = "Dummy task";
        const TASK_EVENT1: EventMask = 0x00000001;
        const TASK_EVENT2: EventMask = 0x00000002;

        let mut scheduler: Scheduler<TASK_COUNT, DUMMY_CORE_FREQ> = Scheduler::new();
        let task = Task::new(TASK_NAME, None, None, None, None);

        let task_event_mask = scheduler.get_task_event(TASK_NAME);
        assert_eq!(task_event_mask, None);

        scheduler.add_task(task);

        let task_event_mask = scheduler.get_task_event(TASK_NAME);
        assert_eq!(task_event_mask, Some(0));

        scheduler.set_task_event(TASK_NAME, TASK_EVENT1 | TASK_EVENT2);
        let task_event_mask = scheduler.get_task_event(TASK_NAME);
        assert_eq!(task_event_mask, Some(TASK_EVENT1 | TASK_EVENT2));
    }

    #[test]
    #[should_panic]
    fn task_name_duplication() {
        const TASK_COUNT: usize = 2;
        let mut scheduler: Scheduler<TASK_COUNT, DUMMY_CORE_FREQ> = Scheduler::new();
        let task1 = Task::new("Dummy task 1", None, None, None, None);
        let task2 = Task::new("Dummy task 1", None, None, None, None);

        scheduler.add_task(task1);
        scheduler.add_task(task2);
    }

    #[test]
    #[should_panic]
    fn task_init_runnable_duplication() {
        const TASK_COUNT: usize = 2;
        fn dummy_init_runnable() {}

        let mut scheduler: Scheduler<TASK_COUNT, DUMMY_CORE_FREQ> = Scheduler::new();
        let task1 = Task::new("Dummy task 1", Some(dummy_init_runnable), None, None, None);
        let task2 = Task::new("Dummy task 2", Some(dummy_init_runnable), None, None, None);

        scheduler.add_task(task1);
        scheduler.add_task(task2);
    }

    #[test]
    #[should_panic]
    fn task_process_runnable_duplication() {
        const TASK_COUNT: usize = 2;
        fn dummy_process_runnable(_event_mask: EventMask) {}

        let mut scheduler: Scheduler<TASK_COUNT, DUMMY_CORE_FREQ> = Scheduler::new();
        let task1 = Task::new(
            "Dummy task 1",
            None,
            Some(dummy_process_runnable),
            None,
            None,
        );
        let task2 = Task::new(
            "Dummy task 2",
            None,
            Some(dummy_process_runnable),
            None,
            None,
        );

        scheduler.add_task(task1);
        scheduler.add_task(task2);
    }
}
