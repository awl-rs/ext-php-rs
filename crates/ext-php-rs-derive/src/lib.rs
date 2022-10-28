use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn php_class(args: TokenStream, input: TokenStream) -> TokenStream {
    macros_impl::php_class(args.into(), input.into()).into()
}
#[proc_macro_attribute]
pub fn php_function(args: TokenStream, input: TokenStream) -> TokenStream {
    macros_impl::php_function(args.into(), input.into()).into()
}
#[proc_macro_attribute]
pub fn php_module(args: TokenStream, input: TokenStream) -> TokenStream {
    macros_impl::php_module(args.into(), input.into()).into()
}
#[proc_macro_attribute]
pub fn php_startup(args: TokenStream, input: TokenStream) -> TokenStream {
    macros_impl::php_startup(args.into(), input.into()).into()
}
#[proc_macro_attribute]
pub fn php_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    macros_impl::php_impl(args.into(), input.into()).into()
}
#[proc_macro_attribute]
pub fn php_const(args: TokenStream, input: TokenStream) -> TokenStream {
    macros_impl::php_const(args.into(), input.into()).into()
}
#[proc_macro_attribute]
pub fn php_extern(args: TokenStream, input: TokenStream) -> TokenStream {
    macros_impl::php_extern(args.into(), input.into()).into()
}
#[proc_macro_derive(ZvalConvert)]
pub fn zval_convert_derive(input: TokenStream) -> TokenStream {
    macros_impl::zval_convert_derive(input.into()).into()
}
#[proc_macro]
pub fn zend_fastcall(input: TokenStream) -> TokenStream {
    macros_impl::zend_fastcall(input.into()).into()
}
