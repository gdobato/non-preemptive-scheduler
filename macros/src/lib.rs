//! Macros which deal with unsafe blocks when
//! instantiating a static mutable Scheduler

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, ExprTuple};

#[proc_macro_attribute]
pub fn scheduler_nonpreeptive(arg: TokenStream, _input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(arg as ExprTuple);
    let (task_count, tick_getter) = (&input.elems[0], &input.elems[1]);
    let gen = quote! {
        static mut SCHEDULER: Scheduler<#task_count> = Scheduler::<#task_count>::new(#tick_getter);
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
pub fn scheduler_add_task(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Expr);
    let gen = quote! {
        unsafe {
            SCHEDULER.add_task(#input);
        }
    };
    gen.into()
}

#[proc_macro]
pub fn scheduler_register_idle_runnable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Expr);
    let gen = quote! {
        unsafe {
            SCHEDULER.register_idle_runnable(#input);
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
            SCHEDULER.set_event(#task_name, #event);
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
            SCHEDULER.clear_event(#task_name, #event)
        }
    };
    gen.into()
}

#[proc_macro]
pub fn scheduler_get_event(input: TokenStream) -> TokenStream {
    let task_name = parse_macro_input!(input as Expr);
    let gen = quote! {
        unsafe {
            SCHEDULER.get_event(#task_name)
        }
    };
    gen.into()
}
