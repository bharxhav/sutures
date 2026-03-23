use proc_macro2::TokenStream;
use quote::quote;
use sutures::v1::{BindingTaskType, Bindings, ConstantValue, Suture, TrieNode};
use syn::{LitStr, parse2};

/// Implementation of `load!` — reads a file path and compiles the sutures manifest.
pub(crate) fn load(input: TokenStream) -> syn::Result<TokenStream> {
    let file_path: LitStr = parse2(input)?;
    let path = file_path.value();

    // Read the file at compile time.
    let content = std::fs::read_to_string(&path)
        .map_err(|e| syn::Error::new(file_path.span(), format!("failed to read `{path}`: {e}")))?;

    compile(&content, file_path.span())
}

/// Implementation of `parse!` — parses an inline JSON string.
pub(crate) fn parse(input: TokenStream) -> syn::Result<TokenStream> {
    let json_str: LitStr = parse2(input)?;
    let content = json_str.value();

    compile(&content, json_str.span())
}

/// Shared compile logic — validates the manifest at compile time and emits
/// tokens that construct the compiled sutures.
fn compile(input: &str, span: proc_macro2::Span) -> syn::Result<TokenStream> {
    // Parse and validate the JSON at macro-expansion time.
    let results = sutures::v1::parse(input)
        .map_err(|e| syn::Error::new(span, format!("invalid sutures manifest: {e}")))?;

    // Surface any per-suture-set compilation errors as compile errors.
    let mut errors = Vec::new();
    let mut compiled = Vec::new();
    for result in results {
        match result {
            Ok(s) => compiled.push(s),
            Err(e) => errors.push(e),
        }
    }

    if !errors.is_empty() {
        let msg = errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(syn::Error::new(
            span,
            format!("suture compilation errors: {msg}"),
        ));
    }

    // Emit tokens that construct Vec<Suture> with Cow::Borrowed strings.
    let suture_tokens: Vec<TokenStream> = compiled.iter().map(emit_suture).collect();

    Ok(quote! {
        {
            use ::std::borrow::Cow;
            vec![#(#suture_tokens),*]
        }
    })
}

// ===========================================================================
// Token emitters
// ===========================================================================

fn emit_suture(suture: &Suture) -> TokenStream {
    let id = emit_option_cow(suture.id());
    let name = emit_cow(suture.name());
    let description = emit_option_cow(suture.description());
    let version = emit_option_cow(suture.version());
    let binding = emit_bindings(suture.binding());
    let constants = emit_constants(suture.constants());

    quote! {
        ::sutures::v1::Suture::__comptime(
            #id,
            #name,
            #description,
            #version,
            #binding,
            #constants,
        )
    }
}

fn emit_bindings(bindings: &Bindings) -> TokenStream {
    match bindings {
        Bindings::Request(root) => {
            let root_tokens = emit_trie_node(root);
            quote! { ::sutures::v1::Bindings::Request(#root_tokens) }
        }
        Bindings::Response(root) => {
            let root_tokens = emit_trie_node(root);
            quote! { ::sutures::v1::Bindings::Response(#root_tokens) }
        }
    }
}

fn emit_trie_node(node: &TrieNode) -> TokenStream {
    let key = emit_cow(node.key());
    let binding = emit_binding_task_type(node.binding());
    let targets: Vec<TokenStream> = node.targets().iter().map(|t| emit_cow(t)).collect();
    let children: Vec<TokenStream> = node.children().iter().map(emit_trie_node).collect();

    quote! {
        ::sutures::v1::TrieNode::__comptime(
            #key,
            #binding,
            vec![#(#targets),*],
            vec![#(#children),*],
        )
    }
}

fn emit_binding_task_type(bt: &BindingTaskType) -> TokenStream {
    match bt {
        BindingTaskType::Direct => {
            quote! { ::sutures::v1::BindingTaskType::Direct }
        }
        BindingTaskType::Iterate { start, end, step } => {
            let s = emit_option_i64(start);
            let e = emit_option_i64(end);
            let st = emit_option_i64(step);
            quote! {
                ::sutures::v1::BindingTaskType::Iterate {
                    start: #s,
                    end: #e,
                    step: #st,
                }
            }
        }
        BindingTaskType::IteratePattern {
            pattern,
            start,
            end,
            step,
        } => {
            let pat = emit_cow(pattern);
            let s = emit_option_i64(start);
            let e = emit_option_i64(end);
            let st = emit_option_i64(step);
            quote! {
                ::sutures::v1::BindingTaskType::IteratePattern {
                    pattern: #pat,
                    start: #s,
                    end: #e,
                    step: #st,
                }
            }
        }
    }
}

fn emit_constants(constants: &[(std::borrow::Cow<'static, str>, ConstantValue)]) -> TokenStream {
    let entries: Vec<TokenStream> = constants
        .iter()
        .map(|(key, val)| {
            let k = emit_cow(key);
            let v = emit_constant_value(val);
            quote! { (#k, #v) }
        })
        .collect();

    quote! { vec![#(#entries),*] }
}

// ===========================================================================
// Leaf emitters
// ===========================================================================

fn emit_cow(s: &str) -> TokenStream {
    quote! { Cow::Borrowed(#s) }
}

fn emit_option_cow(s: Option<&str>) -> TokenStream {
    match s {
        Some(v) => {
            let cow = emit_cow(v);
            quote! { ::core::option::Option::Some(#cow) }
        }
        None => quote! { ::core::option::Option::None },
    }
}

fn emit_option_i64(val: &Option<i64>) -> TokenStream {
    match val {
        Some(v) => quote! { ::core::option::Option::Some(#v) },
        None => quote! { ::core::option::Option::None },
    }
}

fn emit_constant_value(val: &ConstantValue) -> TokenStream {
    match val {
        ConstantValue::Null => {
            quote! { ::sutures::v1::ConstantValue::Null }
        }
        ConstantValue::Bool(b) => {
            quote! { ::sutures::v1::ConstantValue::Bool(#b) }
        }
        ConstantValue::Int(i) => {
            quote! { ::sutures::v1::ConstantValue::Int(#i) }
        }
        ConstantValue::Float(f) => {
            quote! { ::sutures::v1::ConstantValue::Float(#f) }
        }
        ConstantValue::String(s) => {
            let v = s.as_ref();
            quote! { ::sutures::v1::ConstantValue::String(Cow::Borrowed(#v)) }
        }
    }
}
