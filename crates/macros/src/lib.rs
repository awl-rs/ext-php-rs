mod class;
mod constant;
mod extern_;
mod fastcall;
pub mod function;
mod helpers;
mod impl_;
mod method;
mod module;
mod startup_function;
mod syn_ext;
mod zval;

use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

use constant::Constant;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use syn::{
    parse2, parse_macro_input, AttributeArgs, DeriveInput, ItemConst, ItemFn, ItemForeignMod,
    ItemImpl, ItemStruct,
};

extern crate proc_macro;

#[derive(Default, Debug)]
struct State {
    functions: Vec<function::Function>,
    classes: HashMap<String, class::Class>,
    constants: Vec<Constant>,
    startup_function: Option<String>,
    built_module: bool,
}

lazy_static::lazy_static! {
    pub(crate) static ref STATE: StateMutex = StateMutex::new();
}

struct StateMutex(Mutex<State>);

impl StateMutex {
    pub fn new() -> Self {
        Self(Mutex::new(Default::default()))
    }

    pub fn lock(&self) -> MutexGuard<State> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}
