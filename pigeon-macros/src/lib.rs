use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derive macro that generates a `{TypeName}State` struct and a `reconstitute` method.
///
/// Apply `#[derive(Reconstitute)]` to a named-field struct to generate:
///
/// 1. A `{TypeName}State` struct with all the same fields, all `pub`
/// 2. A `pub fn reconstitute(state: {TypeName}State) -> Self` associated function
///    that maps every field from the state struct to the entity
///
/// # Example
///
/// ```ignore
/// #[derive(Reconstitute)]
/// pub struct Application {
///     id: ApplicationId,
///     name: String,
///     version: Version,
/// }
/// ```
///
/// Generates:
///
/// ```ignore
/// pub struct ApplicationState {
///     pub id: ApplicationId,
///     pub name: String,
///     pub version: Version,
/// }
///
/// impl Application {
///     pub fn reconstitute(state: ApplicationState) -> Self {
///         Self {
///             id: state.id,
///             name: state.name,
///             version: state.version,
///         }
///     }
/// }
/// ```
///
/// Only named-field structs are supported. Enums and tuple structs will
/// produce a clear compile error.
#[proc_macro_derive(Reconstitute)]
pub fn derive_reconstitute(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let state_name = syn::Ident::new(&format!("{}State", struct_name), struct_name.span());
    let vis = &input.vis;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            Fields::Unnamed(_) => {
                return syn::Error::new_spanned(
                    struct_name,
                    "Reconstitute only supports structs with named fields, not tuple structs",
                )
                .to_compile_error()
                .into();
            }
            Fields::Unit => {
                return syn::Error::new_spanned(
                    struct_name,
                    "Reconstitute only supports structs with named fields, not unit structs",
                )
                .to_compile_error()
                .into();
            }
        },
        Data::Enum(_) => {
            return syn::Error::new_spanned(
                struct_name,
                "Reconstitute only supports structs, not enums",
            )
            .to_compile_error()
            .into();
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(
                struct_name,
                "Reconstitute only supports structs, not unions",
            )
            .to_compile_error()
            .into();
        }
    };

    let state_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! { pub #name: #ty }
    });

    let field_mappings = fields.iter().map(|f| {
        let name = &f.ident;
        quote! { #name: state.#name }
    });

    let expanded = quote! {
        #vis struct #state_name {
            #(#state_fields,)*
        }

        impl #struct_name {
            pub fn reconstitute(state: #state_name) -> Self {
                Self {
                    #(#field_mappings,)*
                }
            }
        }
    };

    expanded.into()
}
