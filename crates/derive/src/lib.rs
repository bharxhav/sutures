use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Fields, parse_macro_input};

mod attributes;

use attributes::FieldAttributes;

// Entry point for `#[derive(Seam)]`. The `attributes(seam)` part tells the
// compiler that `#[seam(...)]` annotations on fields/variants belong to this
// derive and should not trigger "unknown attribute" errors.
#[proc_macro_derive(Seam, attributes(seam))]
pub fn derive_seam(input: TokenStream) -> TokenStream {
    // `parse_macro_input!` converts the raw token stream into syn's structured
    // AST (`DeriveInput`), which gives us the type name, generics, and body.
    let input = parse_macro_input!(input as syn::DeriveInput);
    match derive_seam_inner(input) {
        Ok(ts) => ts.into(),
        // `to_compile_error` turns a syn::Error into a `compile_error!(...)`
        // invocation so the user sees a clear message instead of a panic.
        Err(e) => e.to_compile_error().into(),
    }
}

fn derive_seam_inner(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    // `split_for_impl` gives us the three pieces needed to write a generic
    // impl block: `impl<T>`, `MyStruct<T>`, and `where T: ...`.
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Dispatch based on whether the input is a struct, enum, or union.
    // Each branch returns (is_enum, Vec<TokenStream>) where each entry in
    // the vec is code that constructs one `SeamField` at runtime.
    let (is_enum, entries) = match &input.data {
        Data::Struct(data) => {
            // Only named fields (i.e. `struct Foo { x: i32 }`) are supported
            // tuple structs and unit structs are rejected.
            let fields = match &data.fields {
                Fields::Named(f) => f,
                _ => {
                    return Err(syn::Error::new_spanned(
                        name,
                        "Seam only supports structs with named fields",
                    ));
                }
            };
            (false, derive_struct_fields(fields)?)
        }
        Data::Enum(data) => (true, derive_enum_variants(data)?),
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                name,
                "Seam does not support unions",
            ));
        }
    };

    // `quote!` is a quasi-quoting macro — it produces Rust source code as
    // tokens. The `#variable` syntax splices Rust values in, and `#(#vec),*`
    // repeats over an iterator with commas between items.
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics sutures::Seam for #name #ty_generics #where_clause {
            const IS_ENUM: bool = #is_enum;
            const IS_ANON_STRUCT: bool = false;

            fn fields() -> &'static [sutures::seam::SeamField] {
                // Each `entries` element expands to a `SeamField { ... }` expr.
                &[#(#entries),*]
            }
        }
    })
}

/// Generate a `SeamField` expression for each named field in a struct.
/// Returns `Err` if any `#[seam(...)]` attribute is malformed.
fn derive_struct_fields(fields: &syn::FieldsNamed) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    // `filter_map` lets us skip fields (return None) or propagate errors
    // (return Some(Err(...))). The final `.collect()` short-circuits on the
    // first error thanks to `Result`'s `FromIterator` impl.
    fields
        .named
        .iter()
        .filter_map(|field| {
            // Parse any `#[seam(...)]` attributes on this field.
            let attrs = match FieldAttributes::from_field(field) {
                Ok(a) => a,
                Err(e) => return Some(Err(e)),
            };
            // `#[seam(skip)]` — omit this field from the schema entirely.
            if attrs.skip {
                return None;
            }

            // Use the `#[seam(rename = "...")]` value if present, otherwise
            // fall back to the field's Rust identifier (stripping the `r#`
            // prefix that raw identifiers like `r#type` carry).
            let field_name = attrs.rename.unwrap_or_else(|| {
                let raw = field.ident.as_ref().unwrap().to_string();
                raw.strip_prefix("r#").unwrap_or(&raw).to_owned()
            });

            // Decide which `SeamFieldType` variant to emit. The `to_struct`,
            // `to_enum`, and `to_anon_struct` attrs tell the macro to recurse
            // into that type's `Seam` impl and embed its fields inline.
            let field_ty = &field.ty;
            let ty = if attrs.to_struct {
                // `const _: () = assert!(...)` is a compile-time assertion —
                // it fires during monomorphisation if the inner type's flags
                // don't match the attribute, giving a clear error message.
                quote! {
                    {
                        const _: () = assert!(
                            !<#field_ty as sutures::Seam>::IS_ENUM
                            && !<#field_ty as sutures::Seam>::IS_ANON_STRUCT,
                            "field marked #[seam(to_struct)] but type is not a Seam struct"
                        );
                        sutures::seam::SeamFieldType::Struct(<#field_ty as sutures::Seam>::fields)
                    }
                }
            } else if attrs.to_enum {
                quote! {
                    {
                        const _: () = assert!(
                            <#field_ty as sutures::Seam>::IS_ENUM,
                            "field marked #[seam(to_enum)] but type is not a Seam enum"
                        );
                        sutures::seam::SeamFieldType::Enum(<#field_ty as sutures::Seam>::fields)
                    }
                }
            } else if attrs.to_anon_struct {
                quote! {
                    {
                        const _: () = assert!(
                            <#field_ty as sutures::Seam>::IS_ANON_STRUCT,
                            "field marked #[seam(to_anon_struct)] but type is not a Seam anonymous struct"
                        );
                        sutures::seam::SeamFieldType::AnonymousStruct(<#field_ty as sutures::Seam>::fields)
                    }
                }
            } else {
                // No nesting attribute — this field is a leaf (e.g. String, i32).
                quote! { sutures::seam::SeamFieldType::Terminal }
            };

            Some(Ok(quote! {
                sutures::seam::SeamField {
                    name: #field_name,
                    ty: #ty,
                }
            }))
        })
        .collect()
}

/// Generate a `SeamField` expression for each variant in an enum.
fn derive_enum_variants(data: &syn::DataEnum) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    data.variants
        .iter()
        .filter_map(|variant| {
            let attrs = match FieldAttributes::from_variant(variant) {
                Ok(a) => a,
                Err(e) => return Some(Err(e)),
            };
            if attrs.skip {
                return None;
            }

            let variant_name = attrs.rename.unwrap_or_else(|| {
                let raw = variant.ident.to_string();
                raw.strip_prefix("r#").unwrap_or(&raw).to_owned()
            });

            // The field type depends on the variant shape:
            //   Unit     (`Foo`)            → Terminal (just a tag, no payload)
            //   Named    (`Foo { x: i32 }`) → AnonymousStruct (recurse into fields)
            //   Unnamed  (`Foo(i32)`)       → Terminal (payload is opaque)
            let ty = match &variant.fields {
                Fields::Unit => {
                    quote! { sutures::seam::SeamFieldType::Terminal }
                }
                Fields::Named(named) => {
                    // Reuse the struct field logic to build child SeamFields,
                    // then wrap them in a closure `|| &[...]` so the slice is
                    // only constructed on demand.
                    let children = match derive_struct_fields(named) {
                        Ok(c) => c,
                        Err(e) => return Some(Err(e)),
                    };
                    quote! {
                        sutures::seam::SeamFieldType::AnonymousStruct(|| &[#(#children),*])
                    }
                }
                Fields::Unnamed(_) => {
                    quote! { sutures::seam::SeamFieldType::Terminal }
                }
            };

            Some(Ok(quote! {
                sutures::seam::SeamField {
                    name: #variant_name,
                    ty: #ty,
                }
            }))
        })
        .collect()
}
