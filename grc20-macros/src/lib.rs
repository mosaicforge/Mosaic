mod entity;
mod relation;

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use relation::RelationOpts;
use syn::{parse_macro_input, DeriveInput};

use entity::EntityOpts;

/// Implements the `FromAttributes` and `IntoAttributes` traits for a struct.
///
/// This macro automatically implements the conversion traits between the knowledge graph's triple
/// representation and Rust structs. It supports:
///
/// - Automatic implementation of `FromAttributes` and `IntoAttributes`
/// - Optional fields using `Option<T>`
/// - Field renaming with `#[grc20(attribute = "...")]`
/// - Type tagging with `#[grc20(schema_type = "...")]` (placeholder for future use)
///
/// # Example
///
/// ```rust
/// #[grc20::entity]
/// #[grc20(schema_type = system_ids::PERSON_TYPE)]
/// struct Person {
///     #[grc20(attribute = system_ids::NAME_ATTRIBUTE)]
///     name: String,
///     #[grc20(attribute = system_ids::NICKNAME_ATTRIBUTE)]
///     nickname: Option<String>,
///     #[grc20(attribute = system_ids::AGE_ATTRIBUTE)]
///     age: u64,
/// }
/// ```
#[proc_macro_attribute]
pub fn entity(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let opts = EntityOpts::from_derive_input(&input).expect("Failed to parse input");

    let impl_from_attributes = entity::generate_from_attributes_impl(&opts);
    let impl_into_attributes = entity::generate_into_attributes_impl(&opts);

    input.attrs.retain(|attr| !attr.path().is_ident("grc20"));

    if let syn::Data::Struct(ref mut data_struct) = input.data {
        for field in &mut data_struct.fields {
            field.attrs.retain(|attr| !attr.path().is_ident("grc20"));
        }
    }

    // let impl_builder = entity::generate_builder_impl(&opts);

    quote! {
        #[derive(Debug)]
        #input

        #impl_from_attributes
        #impl_into_attributes
        // #impl_builder
    }
    .into()
}

/// Implements the `FromAttributes` and `IntoAttributes` traits for a struct.
///
/// This macro automatically implements the conversion traits between the knowledge graph's triple
/// representation and Rust structs. It supports:
///
/// - Automatic implementation of `FromAttributes` and `IntoAttributes`
/// - Optional fields using `Option<T>`
/// - Field renaming with `#[grc20(attribute = "...")]`
/// - Type tagging with `#[grc20(schema_type = "...")]` (placeholder for future use)
///
/// # Example
///
/// ```rust
/// #[grc20::relation]
/// #[grc20(relation_type = system_ids::PARENT_SPACE_RELATION)]
/// struct ParentSpace {
///     #[grc20(attribute = system_ids::DATE_ADDED_ATTRIBUTE)]
///     date_added: DateTime<Utc>,
/// }
/// ```
#[proc_macro_attribute]
pub fn relation(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let opts = RelationOpts::from_derive_input(&input).expect("Failed to parse input");

    let impl_from_attributes = relation::generate_from_attributes_impl(&opts);
    let impl_into_attributes = relation::generate_into_attributes_impl(&opts);

    input.attrs.retain(|attr| !attr.path().is_ident("grc20"));

    if let syn::Data::Struct(ref mut data_struct) = input.data {
        for field in &mut data_struct.fields {
            field.attrs.retain(|attr| !attr.path().is_ident("grc20"));
        }
    }

    // let impl_builder = relation::generate_builder_impl(&opts);

    quote! {
        #[derive(Debug)]
        #input

        #impl_from_attributes
        #impl_into_attributes
        // #impl_builder
    }
    .into()
}
