//! Macros which deal with unsafe blocks when
//! instantiating a static mutable Scheduler

use proc_macro::*;
use quote::{quote, ToTokens};
use syn::{parse::*, parse_macro_input, AttributeArgs, Expr, LitStr, Meta, NestedMeta, Token};

struct TaskEvent {
    task_name: LitStr,
    event: Expr,
}

impl Parse for TaskEvent {
    fn parse(input: ParseStream) -> Result<Self> {
        let task_name: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let event: Expr = input.parse()?;
        Ok(TaskEvent { task_name, event })
    }
}

struct Task {
    name: LitStr,
    init_runnable: Expr,
    process_runnable: Expr,
    execution_cycle: Expr,
    execution_offset: Expr,
}

impl Parse for Task {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let init_runnable: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let process_runnable: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let execution_cycle: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let execution_offset: Expr = input.parse()?;
        Ok(Task {
            name,
            init_runnable,
            process_runnable,
            execution_cycle,
            execution_offset,
        })
    }
}

#[proc_macro_attribute]
pub fn scheduler(args: TokenStream, _input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let (mut task_count, mut core_freq) = (None, None);

    for arg in args {
        match arg {
            NestedMeta::Meta(meta) => {
                if let Meta::NameValue(ref name_value) = meta {
                    let name = meta.path().into_token_stream().to_string();
                    match name.as_str() {
                        "task_count" => {
                            task_count = Some(name_value.lit.clone());
                        }
                        "core_freq" => {
                            core_freq = Some(name_value.lit.clone());
                        }
                        _ => panic!("Unrecognized argument: {}", name),
                    }
                }
            }
            _ => panic!("Unexpected argument type"),
        }
    }

    let task_count = task_count.expect("`task_count` argument is required");
    let core_freq = core_freq.expect("`core_freq` argument is required");
    let gen = quote! {
        static mut SCHEDULER: Scheduler<#task_count, #core_freq> = Scheduler::<#task_count, #core_freq>::new();
    };
    gen.into()
}

#[proc_macro]
pub fn scheduler_launch(_input: TokenStream) -> TokenStream {
    let gen = quote! {
        unsafe {
            SCHEDULER.launch()
        }
    };
    gen.into()
}

#[proc_macro]
pub fn add_task(input: TokenStream) -> TokenStream {
    let Task {
        name,
        init_runnable,
        process_runnable,
        execution_cycle,
        execution_offset,
    } = parse_macro_input!(input as Task);
    let gen = quote! {
        let task = Task::new(#name, #init_runnable, #process_runnable, #execution_cycle, #execution_offset);
        unsafe {
            SCHEDULER.add_task(task);
        }
    };
    gen.into()
}

#[proc_macro]
pub fn register_idle_runnable(input: TokenStream) -> TokenStream {
    let idle_runnable = parse_macro_input!(input as Expr);
    let gen = quote! {
        unsafe {
            SCHEDULER.register_idle_runnable(#idle_runnable);
        }
    };
    gen.into()
}

#[proc_macro]
pub fn set_task_event(input: TokenStream) -> TokenStream {
    let TaskEvent { task_name, event } = parse_macro_input!(input as TaskEvent);
    let gen = quote! {
        unsafe {
            SCHEDULER.set_task_event(#task_name, #event);
        }
    };
    gen.into()
}

#[proc_macro]
pub fn clear_task_event(input: TokenStream) -> TokenStream {
    let TaskEvent { task_name, event } = parse_macro_input!(input as TaskEvent);
    let gen = quote! {
        unsafe {
            SCHEDULER.clear_task_event(#task_name, #event)
        }
    };
    gen.into()
}

#[proc_macro]
pub fn get_task_event(input: TokenStream) -> TokenStream {
    let task_name = parse_macro_input!(input as LitStr);
    let gen = quote! {
        unsafe {
            SCHEDULER.get_task_event(#task_name)
        }
    };
    gen.into()
}
