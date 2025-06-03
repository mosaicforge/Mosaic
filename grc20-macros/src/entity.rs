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

pub(crate) fn generate_builder_impl(opts: &EntityOpts) -> TokenStream2 {
    let struct_name = &opts.ident;
    let builder_name = Ident::new(&format!("{}Builder", struct_name), Span::call_site());
    let fields = opts.data.as_ref().take_struct().expect("Expected struct");
    let schema_type = opts.schema_type.as_ref().map(|s| quote!(#s));

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
        let mut_name = Ident::new(&format!("{}_mut", field_name), Span::call_site());
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

    let type_set_block = if let Some(schema_type) = &schema_type {
        quote! {
            built = built.with_type(#schema_type);
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

            pub fn build(self) -> grc20_core::mapping::Entity<#struct_name> {
                let mut built = grc20_core::Entity::new(
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

pub(crate) fn generate_query_impls(opts: &EntityOpts) -> TokenStream2 {
    let struct_name = &opts.ident;
    let fields = opts.data.as_ref().take_struct().expect("Expected struct");

    let find_one_fn = quote! {
        /// Find a person by its id
        pub fn find_one(
            neo4j: &grc20_core::neo4rs::Graph,
            id: impl Into<String>,
            space_id: impl Into<String>,
        ) -> FindOneQuery {
            FindOneQuery::new(neo4j.clone(), id.into(), space_id.into())
        }
    };

    let find_many_fn = quote! {
        /// Find multiple persons with filters
        pub fn find_many(neo4j: &grc20_core::neo4rs::Graph, space_id: impl Into<String>) -> FindManyQuery {
            FindManyQuery::new(neo4j.clone(), space_id.into())
        }
    };

    let find_one_query_struct = quote! {
        /// Query to find a single person
        pub struct FindOneQuery {
            neo4j: grc20_core::neo4rs::Graph,
            id: String,
            space_id: String,
            version: Option<String>,
        }

        impl FindOneQuery {
            fn new(neo4j: grc20_core::neo4rs::Graph, id: String, space_id: String) -> Self {
                Self {
                    neo4j,
                    id,
                    space_id,
                    version: None,
                }
            }

            pub fn version(mut self, version: impl Into<String>) -> Self {
                self.version = Some(version.into());
                self
            }

            pub fn version_opt(mut self, version: Option<String>) -> Self {
                self.version = version;
                self
            }
        }

        impl grc20_core::mapping::query_utils::Query<Option<grc20_core::mapping::Entity<#struct_name>>> for FindOneQuery {
            async fn send(self) -> Result<Option<grc20_core::mapping::Entity<#struct_name>>, grc20_core::error::DatabaseError> {
                grc20_core::entity::find_one::<grc20_core::mapping::Entity<#struct_name>>(
                    &self.neo4j,
                    self.id,
                )
                    .space_id(self.space_id)
                    .version_opt(self.version)
                    .send()
                    .await
            }
        }
    };

    // Collect field names and types
    let field_names: Vec<_> = fields
        .iter()
        .map(|f| f.ident.as_ref().expect("Expected named field"))
        .collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    // Generate fields for FindManyQuery
    let find_many_fields = field_names.iter().zip(field_types.iter())
        .map(|(field_name, field_type)| {
            if let syn::Type::Path(type_path) = field_type {
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
                            return quote! {
                                #field_name: Option<grc20_core::mapping::query_utils::PropFilter<#inner_type>>,
                            };
                        }
                    }
                }
            }
            quote! {
                #field_name: Option<grc20_core::mapping::query_utils::PropFilter<#field_type>>,
            }
        })
        .collect::<Vec<_>>();

    // Generate builder methods for FindManyQuery
    let find_many_methods = field_names
        .iter()
        .zip(field_types.iter())
        .map(|(field_name, field_type)| {
            let doc_comment = format!("Filter by {}", field_name);
            let filter_type = if let syn::Type::Path(type_path) = field_type {
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
                            quote!(grc20_core::mapping::query_utils::PropFilter<#inner_type>)
                        } else {
                            panic!("Expected inner type for Option<T>")
                        }
                    } else {
                        panic!("Expected angle-bracketed arguments for Option<T>")
                    }
                } else {
                    quote!(grc20_core::mapping::query_utils::PropFilter<#field_type>)
                }
            } else {
                quote!(grc20_core::mapping::query_utils::PropFilter<#field_type>)
            };

            quote! {
                #[doc = #doc_comment]
                pub fn #field_name(mut self, #field_name: #filter_type) -> Self {
                self.#field_name = Some(#field_name);
                self
                }
            }
        })
        .collect::<Vec<_>>();

    // Generate attribute filter applications for QueryStream implementation
    let find_many_filters = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().expect("Expected named field");
            let attribute_name = field
                .attribute
                .as_ref()
                .map(|s| quote!(#s))
                .unwrap_or_else(|| quote!(#field_name.to_string()));

            quote! {
                if let Some(#field_name) = self.#field_name {
                    query = query.attribute(
                        grc20_core::mapping::query_utils::AttributeFilter::new(#attribute_name)
                            .value(#field_name.as_string())
                    );
                }
            }
        })
        .collect::<Vec<_>>();

    let schema_type = opts.schema_type.as_ref().map(|s| quote!(#s));
    let type_filter = if let Some(schema_type) = schema_type {
        quote! {
            .relations(grc20_core::mapping::entity::TypesFilter::default().r#type(#schema_type.to_string()))
        }
    } else {
        quote! {}
    };

    let find_many_query_struct = quote! {
        /// Query to find multiple persons with filters
        pub struct FindManyQuery {
            neo4j: grc20_core::neo4rs::Graph,
            id: Option<grc20_core::mapping::query_utils::PropFilter<String>>,
            #(#find_many_fields)*
            space_id: String,
            version: Option<String>,
            limit: usize,
            skip: Option<usize>,
        }

        impl FindManyQuery {
            fn new(neo4j: grc20_core::neo4rs::Graph, space_id: String) -> Self {
                let mut query = Self {
                    neo4j,
                    id: None,
                    #(
                        #field_names: None,
                    )*
                    space_id,
                    version: None,
                    limit: 100,
                    skip: None,
                };

                query
            }

            pub fn id(mut self, id: grc20_core::mapping::query_utils::PropFilter<String>) -> Self {
                self.id = Some(id);
                self
            }

            #(#find_many_methods)*

            pub fn version(mut self, version: impl Into<String>) -> Self {
                self.version = Some(version.into());
                self
            }

            pub fn version_opt(mut self, version: Option<String>) -> Self {
                self.version = version;
                self
            }

            /// Limit the number of results
            pub fn limit(mut self, limit: usize) -> Self {
                self.limit = limit;
                self
            }

            /// Skip a number of results
            pub fn skip(mut self, skip: usize) -> Self {
                self.skip = Some(skip);
                self
            }
        }

        impl grc20_core::mapping::query_utils::QueryStream<grc20_core::mapping::Entity<#struct_name>> for FindManyQuery {
            async fn send(
                self,
            ) -> Result<impl futures::Stream<Item = Result<grc20_core::mapping::Entity<#struct_name>, grc20_core::error::DatabaseError>>, grc20_core::error::DatabaseError> {
                let mut query = grc20_core::entity::find_many::<grc20_core::mapping::Entity<#struct_name>>(
                    &self.neo4j,
                )
                    .space_id(self.space_id)
                    .version_opt(self.version)
                    .limit(self.limit)
                    .with_filter(
                        grc20_core::mapping::EntityFilter::default()
                            #type_filter
                    );

                #(#find_many_filters)*

                if let Some(skip) = self.skip {
                    query = query.skip(skip);
                }

                query.send().await
            }
        }
    };

    quote! {
        use grc20_core::{
            mapping::{
                query_utils::{Query as _, QueryStream as __},
            },
        };

        #find_one_fn
        #find_many_fn
        #find_one_query_struct
        #find_many_query_struct
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
