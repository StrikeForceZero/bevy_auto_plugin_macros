#![cfg_attr(feature = "nightly_proc_macro_span", feature(proc_macro_span))]
use proc_macro2::Span;
use quote::quote;
use bevy_auto_plugin_shared::util::{path_to_string, Target};
use bevy_auto_plugin_shared::AutoPluginContext;
use std::cell::RefCell;
use std::collections::HashMap;
use syn::Path;
use thiserror::Error;

thread_local! {
    static FILE_STATE_MAP: RefCell<HashMap<String, FileState>> = RefCell::new(HashMap::new());
}

// TODO: is there a better way? this originally was using Path instead of String
//  but apparently static references to Path creates "use after free" errors
#[derive(Default)]
pub struct FileState {
    pub plugin_registered: bool,
    pub context: AutoPluginContext,
}

pub fn get_file_path() -> String {
    #[cfg(feature = "nightly_proc_macro_span")]
    let file_path = Span::call_site()
        .unwrap()
        .source_file()
        .path()
        .display()
        .to_string();

    #[cfg(not(feature = "nightly_proc_macro_span"))]
    let file_path = {
        panic!("proc_macro_span feature is required for this crate");
    };
    file_path
}

pub fn update_file_state<R>(file_path: String, update_fn: impl FnOnce(&mut FileState) -> R) -> R {
    FILE_STATE_MAP.with(|map| {
        let mut map = map.borrow_mut();
        let file_state = map.entry(file_path).or_default();
        update_fn(file_state)
    })
}

pub fn update_state(
    file_path: String,
    path: Path,
    target: Target,
) -> std::result::Result<(), UpdateStateError> {
    FILE_STATE_MAP.with(|map| {
        let mut map = map.borrow_mut();
        let entry = map.entry(file_path).or_default();
        if entry.plugin_registered {
            return Err(UpdateStateError::PluginAlreadyRegistered);
        }
        let path = path_to_string(&path, false);
        let inserted = match target {
            Target::RegisterTypes => entry.context.register_types.insert(path),
            Target::AddEvents => entry.context.add_events.insert(path),
            Target::InitResources => entry.context.init_resources.insert(path),
            Target::RequiredComponentAutoName => entry.context.auto_names.insert(path),
        };
        if !inserted {
            return Err(UpdateStateError::Duplicate);
        }
        Ok(())
    })
}

fn get_files_missing_plugin() -> Vec<String> {
    FILE_STATE_MAP.with(|map| {
        let map = map.borrow();
        let mut files_missing_plugin = Vec::new();
        for (file_path, file_state) in map.iter() {
            if file_state.plugin_registered {
                continue;
            }
            files_missing_plugin.push(file_path.clone());
        }
        files_missing_plugin
    })
}

pub fn files_missing_plugin_ts() -> proc_macro2::TokenStream {
    #[allow(unused_mut)]
    let mut output = quote! {};
    let missing_plugin_files = get_files_missing_plugin();
    if !missing_plugin_files.is_empty() {
        #[allow(unused_variables)]
        let messages = missing_plugin_files
            .into_iter()
            .map(|file_path| format!("missing #[auto_plugin(...)] attribute in file: {file_path}"))
            .collect::<Vec<_>>();
        #[cfg(feature = "missing_auto_plugin_is_error")]
        {
            output.extend(messages.iter().map(|message| {
                quote! {
                    log::error!(#message);
                }
            }));
        }
        #[cfg(feature = "missing_auto_plugin_is_warning")]
        {
            output.extend(messages.iter().map(|message| {
                quote! {
                    log::warn!(#message);
                }
            }));
        }
        #[cfg(feature = "missing_auto_plugin_is_compile_error")]
        return syn::Error::new(Span::call_site(), messages.join("\n")).to_compile_error();
    }
    output
}

#[derive(Error, Debug)]
pub enum UpdateStateError {
    #[error("duplicate attribute")]
    Duplicate,
    #[error("plugin already registered above, move plugin fn to the bottom of the file")]
    PluginAlreadyRegistered,
}
