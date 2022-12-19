//! Macros which deal with unsafe blocks when
//! instantiating a static mutable Scheduler

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, ExprTuple};

#[proc_macro_attribute]
pub fn scheduler_nonpreeptive(arg: TokenStream, _input: TokenStream) -> TokenStream {
    let task_count = parse_macro_input!(arg as Expr);
    let gen = quote! {
        const TASK_COUNT: usize = #task_count;
        static mut SCHEDULER: Option<Scheduler<TASK_COUNT>> = None;
    };
    gen.into()
}

#[proc_macro]
pub fn scheduler_init(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Expr);
    let gen = quote! {
        unsafe {
            SCHEDULER.replace(Scheduler::<TASK_COUNT>::new(#input));
        }
    };
    gen.into()
}

#[proc_macro]
pub fn scheduler_launch(_input: TokenStream) -> TokenStream {
    let gen = quote! {
        unsafe {
            if let Some(scheduler) = &mut SCHEDULER {
                scheduler.launch();
            }
        }
    };
    gen.into()
}

#[proc_macro]
pub fn scheduler_add_task(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Expr);
    let gen = quote! {
        unsafe {
            if let Some(scheduler) = &mut SCHEDULER {
                scheduler.add_task(#input);
            }
        }
    };
    gen.into()
}

#[proc_macro]
pub fn scheduler_register_idle_runnable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Expr);
    let gen = quote! {
        unsafe {
            if let Some(scheduler) = &mut SCHEDULER {
                scheduler.register_idle_runnable(#input);
            }
        }
    };
    gen.into()
}

#[proc_macro]
pub fn scheduler_set_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ExprTuple);
    let (task_name, event) = (&input.elems[0], &input.elems[1]);
    let gen = quote! {
        unsafe {
            if let Some(scheduler) = &mut SCHEDULER {
               scheduler.set_event(#task_name, #event);
            }
        }
    };
    gen.into()
}

#[proc_macro]
pub fn scheduler_clear_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ExprTuple);
    let (task_name, event) = (&input.elems[0], &input.elems[1]);
    let gen = quote! {
        unsafe {
            if let Some(scheduler) = &mut SCHEDULER {
                scheduler.clear_event(#task_name, #event)
            }
        }
    };
    gen.into()
}

#[proc_macro]
pub fn scheduler_get_event(input: TokenStream) -> TokenStream {
    let task_name = parse_macro_input!(input as Expr);
    let gen = quote! {
        unsafe {
            if let Some(scheduler) = &mut SCHEDULER {
                scheduler.get_event(#task_name)
            } else {
                None
            }
        }
    };
    gen.into()
}
