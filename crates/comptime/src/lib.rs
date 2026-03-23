mod v1;

use proc_macro::TokenStream;

/// Compile-time equivalent of [`sutures::v1::load`].
///
/// Reads and compiles a `.sutures.json` file at compile time.
///
/// ```ignore
/// let sutures = sutures_comptime::load!("path/to/file.sutures.json");
/// ```
#[proc_macro]
pub fn load(input: TokenStream) -> TokenStream {
    v1::load(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Compile-time equivalent of [`sutures::v1::parse`].
///
/// Parses and compiles an inline JSON string at compile time.
///
/// ```ignore
/// let sutures = sutures_comptime::parse!(r#"{ "name": "Foo", "suture_sets": [...] }"#);
/// ```
#[proc_macro]
pub fn parse(input: TokenStream) -> TokenStream {
    v1::parse(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
