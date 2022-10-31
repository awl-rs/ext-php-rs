pub mod class;
mod constant;
mod extern_;
mod fastcall;
pub mod function;
mod helpers;
pub mod impl_;
mod method;
pub mod module;
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
use syn::{parse::Parser, parse2, punctuated::Punctuated, AttributeArgs, NestedMeta, Token};

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

pub fn php_class(args: TokenStream, input: TokenStream) -> TokenStream {
    let parser = Punctuated::<NestedMeta, Token![,]>::parse_terminated;
    let args: AttributeArgs = parser.parse2(args).unwrap().into_iter().collect();
    let input = parse2(input).unwrap();

    match class::parser(args, input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

pub fn php_function(args: TokenStream, input: TokenStream) -> TokenStream {
    let parser = Punctuated::<NestedMeta, Token![,]>::parse_terminated;
    let args: AttributeArgs = parser.parse2(args).unwrap().into_iter().collect();
    let input = parse2(input).unwrap();

    match function::parser(args, input) {
        Ok((parsed, _)) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

pub fn php_module(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse2(input).unwrap();

    match module::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

pub fn php_startup(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse2(input).unwrap();

    match startup_function::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

pub fn php_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let parser = Punctuated::<NestedMeta, Token![,]>::parse_terminated;
    let args: AttributeArgs = parser.parse2(args).unwrap().into_iter().collect();
    let input = parse2(input).unwrap();

    match impl_::parser(args, input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

pub fn php_const(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse2(input).unwrap();

    match constant::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

pub fn php_extern(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse2(input).unwrap();

    match extern_::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

pub fn zval_convert_derive(input: TokenStream) -> TokenStream {
    let input = parse2(input).unwrap();

    match zval::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}

pub fn zend_fastcall(input: TokenStream) -> TokenStream {
    let input = parse2(input).unwrap();

    match fastcall::parser(input) {
        Ok(parsed) => parsed,
        Err(e) => syn::Error::new(Span::call_site(), e).to_compile_error(),
    }
    .into()
}
