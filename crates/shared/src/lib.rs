use proc_macro2::{Ident, TokenStream as MacroStream};
use quote::quote;
use std::collections::HashSet;
use syn::Path;

pub mod util;

#[derive(Default)]
pub struct AutoPluginContext {
    pub register_types: HashSet<String>,
    pub add_events: HashSet<String>,
    pub init_resources: HashSet<String>,
}

pub fn generate_register_types(
    app_ident: &Ident,
    items: impl Iterator<Item = String>,
) -> syn::Result<MacroStream> {
    let register_types = items
        .map(|item| {
            let item = syn::parse_str::<Path>(&item)?;
            Ok(quote! {
                #app_ident.register_type::<#item>();
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    Ok(quote! {
        {
            // register_types
            #(#register_types)*
        }
    })
}

pub fn generate_add_events(
    app_ident: &Ident,
    items: impl Iterator<Item = String>,
) -> syn::Result<MacroStream> {
    let add_events = items
        .map(|item| {
            let item = syn::parse_str::<Path>(&item)?;
            Ok(quote! {
                #app_ident.add_event::<#item>();
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    Ok(quote! {
        {
            // add_events
            #(#add_events)*
        }
    })
}

pub fn generate_init_resources(
    app_ident: &Ident,
    items: impl Iterator<Item = String>,
) -> syn::Result<MacroStream> {
    let init_resources = items
        .map(|item| {
            let item = syn::parse_str::<Path>(&item)?;
            Ok(quote! {
                #app_ident.init_resource::<#item>();
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    Ok(quote! {
        {
            // init_resources
            #(#init_resources)*
        }
    })
}
