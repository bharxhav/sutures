use syn::LitStr;

/// Parsed representation of `#[seam(...)]` helper attributes.
///
/// These attributes control how the `#[derive(Seam)]` macro maps Rust types to
/// the Seam schema. They can appear on struct fields or enum variants:
///
/// ```ignore
/// #[derive(Seam)]
/// struct Config {
///     name: String,                        // Terminal — no attribute needed
///     #[seam(rename = "addr")]
///     address: String,                     // Terminal, exposed as "addr"
///     #[seam(skip)]
///     internal: u64,                       // Omitted from the schema
///     #[seam(to_struct)]
///     nested: Inner,                       // Recurse into Inner's Seam impl
/// }
/// ```
pub(crate) struct FieldAttributes {
    /// Override the field/variant name in the generated schema.
    pub rename: Option<String>,
    /// Exclude this field/variant from the schema entirely.
    pub skip: bool,
    /// Embed the target type's fields inline as a nested struct.
    pub to_struct: bool,
    /// Embed the target type's variants inline as a nested enum.
    pub to_enum: bool,
    /// Embed the target type's fields as an anonymous (unnamed) struct.
    pub to_anon_struct: bool,
}

impl FieldAttributes {
    /// Parse `#[seam(...)]` on an enum variant.
    ///
    /// Variants only support `rename` and `skip` — the nesting attributes
    /// (`to_struct`, `to_enum`, `to_anon_struct`) are not available here
    /// because variant payloads are handled structurally in `derive_enum_variants`.
    pub fn from_variant(variant: &syn::Variant) -> syn::Result<Self> {
        let mut attrs = FieldAttributes {
            rename: None,
            skip: false,
            to_struct: false,
            to_enum: false,
            to_anon_struct: false,
        };

        for attr in &variant.attrs {
            // Skip non-`#[seam(...)]` attributes (e.g. `#[serde(...)]`).
            if !attr.path().is_ident("seam") {
                continue;
            }
            // `parse_nested_meta` walks the comma-separated items inside
            // `#[seam(...)]`, calling our closure once per item.
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    if attrs.rename.is_some() {
                        return Err(meta.error("duplicate `rename` attribute"));
                    }
                    // `meta.value()` consumes the `=` and returns a parser
                    // positioned at the value token, which we parse as a string.
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    attrs.rename = Some(s.value());
                } else if meta.path.is_ident("skip") {
                    attrs.skip = true;
                } else {
                    return Err(meta.error("unknown seam variant attribute"));
                }
                Ok(())
            })?;
        }

        Ok(attrs)
    }

    /// Parse `#[seam(...)]` on a struct field.
    ///
    /// Fields support the full attribute set: `rename`, `skip`, and the three
    /// mutually-exclusive nesting attributes (`to_struct`, `to_enum`,
    /// `to_anon_struct`).
    pub fn from_field(field: &syn::Field) -> syn::Result<Self> {
        let mut attrs = FieldAttributes {
            rename: None,
            skip: false,
            to_struct: false,
            to_enum: false,
            to_anon_struct: false,
        };

        for attr in &field.attrs {
            if !attr.path().is_ident("seam") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    if attrs.rename.is_some() {
                        return Err(meta.error("duplicate `rename` attribute"));
                    }
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    attrs.rename = Some(s.value());
                } else if meta.path.is_ident("skip") {
                    attrs.skip = true;
                } else if meta.path.is_ident("to_struct") {
                    attrs.to_struct = true;
                } else if meta.path.is_ident("to_enum") {
                    attrs.to_enum = true;
                } else if meta.path.is_ident("to_anon_struct") {
                    attrs.to_anon_struct = true;
                } else {
                    return Err(meta.error("unknown seam attribute"));
                }
                Ok(())
            })?;
        }

        // Ensure at most one nesting direction is specified — combining them
        // would be ambiguous (is the field a struct or an enum?).
        let type_count = [attrs.to_struct, attrs.to_enum, attrs.to_anon_struct]
            .iter()
            .filter(|&&b| b)
            .count();
        if type_count > 1 {
            return Err(syn::Error::new_spanned(
                &field.ident,
                "only one of to_struct, to_enum, to_anon_struct may be specified",
            ));
        }
        // `skip` + nesting is contradictory: you can't both omit a field and
        // recurse into it.
        if attrs.skip && (attrs.to_struct || attrs.to_enum || attrs.to_anon_struct) {
            return Err(syn::Error::new_spanned(
                &field.ident,
                "skip cannot be combined with to_struct, to_enum, or to_anon_struct",
            ));
        }

        Ok(attrs)
    }
}
