//! Procedural macros for the GRC20 SDK
//! 
//! This crate provides macros for automatically implementing traits from the GRC20 SDK.

use darling::{FromDeriveInput, FromField, FromMeta};
use proc_macro2::{TokenStream as TokenStream2, Span};
use quote::{quote, ToTokens};
use syn::{Ident, Type, Path, LitStr};

/// Represents either a string literal or a path to a constant
#[derive(Debug, Clone)]
pub(crate) enum StringOrPath {
    String(String),
    Path(Path),
}

impl darling::FromMeta for StringOrPath {
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(StringOrPath::String(value.into()))
    }

    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        match value {
            syn::Lit::Str(s) => Ok(StringOrPath::String(s.value())),
            _ => Err(darling::Error::unexpected_lit_type(value)),
        }
    }

    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        match expr {
            syn::Expr::Path(path) => Ok(StringOrPath::Path(path.path.clone())),
            syn::Expr::Lit(lit) => Self::from_value(&lit.lit),
            _ => Err(darling::Error::custom("expected string literal or path expression")),
        }
    }

    fn from_meta(item: &syn::Meta) -> darling::Result<Self> {
        match item {
            syn::Meta::Path(path) => Ok(StringOrPath::Path(path.clone())),
            syn::Meta::NameValue(name_value) => Self::from_expr(&name_value.value),
            _ => Err(darling::Error::custom("expected string literal or path")),
        }
    }
}

impl ToTokens for StringOrPath {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            StringOrPath::String(s) => LitStr::new(s, Span::call_site()).to_tokens(tokens),
            StringOrPath::Path(p) => p.to_tokens(tokens),
        }
    }
}

#[derive(FromDeriveInput)]
#[darling(attributes(grc20), forward_attrs(allow, doc, cfg))]
pub(crate) struct EntityOpts {
    ident: Ident,
    #[darling(default)]
    schema_type: Option<StringOrPath>,
    data: darling::ast::Data<(), EntityFieldOpts>,
}

#[derive(FromField)]
#[darling(attributes(grc20))]
pub(crate) struct EntityFieldOpts {
    ident: Option<Ident>,
    ty: Type,
    #[darling(default)]
    attribute: Option<StringOrPath>,
}

pub(crate) fn generate_from_attributes_impl(opts: &EntityOpts) -> TokenStream2 {
    let struct_name = &opts.ident;
    let fields = opts.data.as_ref().take_struct().expect("Expected struct");

    let field_assignments = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Expected named field");
        let field_type = &field.ty;
        let attribute_name = field
            .attribute
            .as_ref()
            .map(|s| quote!(#s))
            .unwrap_or_else(|| quote!(#field_name.to_string()));

        // Check if field type is Option<T>
        if let syn::Type::Path(type_path) = field_type {
            if type_path.path.segments.last().map(|s| s.ident == "Option").unwrap_or(false) {
                return quote! {
                    #field_name: attributes.pop_opt(#attribute_name)?,
                };
            }
        }

        quote! {
            #field_name: attributes.pop(#attribute_name)?,
        }
    });

    quote! {
        impl sdk::mapping::FromAttributes for #struct_name {
            fn from_attributes(
                mut attributes: sdk::mapping::Attributes,
            ) -> Result<Self, sdk::mapping::TriplesConversionError> {
                Ok(Self {
                    #(#field_assignments)*
                })
            }
        }
    }
}

pub(crate) fn generate_into_attributes_impl(opts: &EntityOpts) -> TokenStream2 {
    let struct_name = &opts.ident;
    let fields = opts.data.as_ref().take_struct().expect("Expected struct");

    let field_conversions = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Expected named field");
        let attribute_name = field
            .attribute
            .as_ref()
            .map(|s| quote!(#s))
            .unwrap_or_else(|| quote!(#field_name.to_string()));

        // Check if field type is Option<T>
        if let syn::Type::Path(type_path) = &field.ty {
            if type_path.path.segments.last().map(|s| s.ident == "Option").unwrap_or(false) {
                return quote! {
                    if let Some(value) = self.#field_name {
                        attributes = attributes.attribute((#attribute_name, value));
                    }
                };
            }
        }

        quote! {
            attributes = attributes.attribute((#attribute_name, self.#field_name));
        }
    });

    quote! {
        impl sdk::mapping::IntoAttributes for #struct_name {
            fn into_attributes(
                self,
            ) -> Result<sdk::mapping::Attributes, sdk::mapping::TriplesConversionError> {
                let mut attributes = sdk::mapping::Attributes::default();
                #(#field_conversions)*
                Ok(attributes)
            }
        }
    }
}
