//! Procedural macros for the GRC20 SDK
//!
//! This crate provides macros for automatically implementing traits from the GRC20 SDK.

use darling::{FromDeriveInput, FromField};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{Ident, LitStr, Path, Type};

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
            _ => Err(darling::Error::custom(
                "expected string literal or path expression",
            )),
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
pub(crate) struct RelationOpts {
    ident: Ident,
    #[darling(default)]
    relation_type: Option<StringOrPath>,
    data: darling::ast::Data<(), RelationFieldOpts>,
}

#[derive(FromField)]
#[darling(attributes(grc20))]
pub(crate) struct RelationFieldOpts {
    ident: Option<Ident>,
    ty: Type,
    #[darling(default)]
    attribute: Option<StringOrPath>,
}

pub(crate) fn generate_from_attributes_impl(opts: &RelationOpts) -> TokenStream2 {
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
            if type_path
                .path
                .segments
                .last()
                .map(|s| s.ident == "Option")
                .unwrap_or(false)
            {
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
        impl grc20_core::mapping::FromAttributes for #struct_name {
            fn from_attributes(
                mut attributes: grc20_core::mapping::Attributes,
            ) -> Result<Self, grc20_core::mapping::TriplesConversionError> {
                Ok(Self {
                    #(#field_assignments)*
                })
            }
        }
    }
}

pub(crate) fn generate_builder_impl(opts: &RelationOpts) -> TokenStream2 {
    let struct_name = &opts.ident;
    let builder_name = Ident::new(&format!("{struct_name}Builder"), Span::call_site());
    let fields = opts.data.as_ref().take_struct().expect("Expected struct");
    let relation_type = opts.relation_type.as_ref().map(|s| quote!(#s));

    // Generate field definitions for the builder
    let builder_fields = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Expected named field");
        let field_type = &field.ty;
        quote! {
            #field_name: #field_type,
        }
    });

    // Generate setter methods for each field
    let setter_methods = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Expected named field");
        let mut_name = Ident::new(&format!("{field_name}_mut"), Span::call_site());
        let field_type = &field.ty;

        // For Option<T> types, we don't need impl Into
        let (param_type, conversion) = if let syn::Type::Path(type_path) = field_type {
            if type_path
                .path
                .segments
                .last()
                .map(|s| s.ident == "Option")
                .unwrap_or(false)
            {
                if let syn::PathArguments::AngleBracketed(args) =
                    &type_path.path.segments.last().unwrap().arguments
                {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        (quote!(#inner_type), quote!(Some(value)))
                    } else {
                        panic!("Expected inner type for Option<T>")
                    }
                } else {
                    (quote!(#field_type), quote!(value))
                }
            } else if type_path
                .path
                .segments
                .last()
                .map(|s| s.ident == "String")
                .unwrap_or(false)
            {
                (quote!(impl Into<String>), quote!(value.into()))
            } else {
                (quote!(#field_type), quote!(value))
            }
        } else {
            // (quote!(#field_type), quote!(value))
            panic!("Expected Option<T> type")
        };

        quote! {
            pub fn #field_name(mut self, value: #param_type) -> Self {
                self.#field_name = #conversion;
                self
            }

            pub fn #mut_name(&mut self, value: #param_type) {
                self.#field_name = #conversion;
            }
        }
    });

    let field_names: Vec<_> = fields
        .iter()
        .map(|f| f.ident.as_ref().expect("Expected named field"))
        .collect();

    let type_set_block = if let Some(relation_type) = &relation_type {
        quote! {
            built = built.with_type(#relation_type);
        }
    } else {
        quote! {}
    };

    quote! {
        impl #struct_name {
            pub fn new(id: impl Into<String>) -> #builder_name {
                #builder_name::new(id.into())
            }
        }

        #[derive(Default)]
        pub struct #builder_name {
            id: String,
            #(#builder_fields)*
        }

        impl #builder_name {
            pub fn new(id: String) -> Self {
                Self {
                    id,
                    ..Default::default()
                }
            }

            #(#setter_methods)*

            pub fn build(self) -> grc20_core::mapping::Relation<#struct_name> {
                let mut built = grc20_core::mapping::Relation::new(
                    self.id,
                    #struct_name {
                        #(
                            #field_names: self.#field_names,
                        )*
                    }
                );

                #type_set_block
                built
            }
        }
    }
}

pub(crate) fn generate_into_attributes_impl(opts: &RelationOpts) -> TokenStream2 {
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
            if type_path
                .path
                .segments
                .last()
                .map(|s| s.ident == "Option")
                .unwrap_or(false)
            {
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
        impl grc20_core::mapping::IntoAttributes for #struct_name {
            fn into_attributes(
                self,
            ) -> Result<grc20_core::mapping::Attributes, grc20_core::mapping::TriplesConversionError> {
                let mut attributes = grc20_core::mapping::Attributes::default();
                #(#field_conversions)*
                Ok(attributes)
            }
        }
    }
}
