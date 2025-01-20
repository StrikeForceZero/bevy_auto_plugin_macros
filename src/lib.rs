#![feature(proc_macro_span)]

use proc_macro::TokenStream as CompilerStream;
use proc_macro2::TokenStream as MacroStream;

use proc_macro2::{Ident, Span};
use quote::quote;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use syn::meta::ParseNestedMeta;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{
    parse_macro_input, Error, FnArg, Generics, Item, ItemFn, Pat, Path, PathArguments, PathSegment,
    Result, Token, Type, TypeReference,
};
use thiserror::Error;

thread_local! {
    static FILE_STATE_MAP: RefCell<HashMap<String, FileState>> = RefCell::new(HashMap::new());
}

// TODO: is there a better way? this originally was using Path instead of String
//  but apparently static references to Path creates "use after free" errors
#[derive(Default)]
struct FileState {
    plugin_registered: bool,
    register_types: HashSet<String>,
    add_events: HashSet<String>,
    init_resources: HashSet<String>,
}

#[derive(Default)]
struct AutoPluginAttributes {
    app_param_name: Option<Ident>,
}

impl AutoPluginAttributes {
    fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        if meta.path.is_ident("app") {
            self.app_param_name = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("unsupported attribute"))
        }
    }
}

#[proc_macro_attribute]
pub fn auto_plugin(attr: CompilerStream, input: CompilerStream) -> CompilerStream {
    let mut attrs = AutoPluginAttributes::default();
    let arg_parser = syn::meta::parser(|meta| attrs.parse(meta));
    parse_macro_input!(attr with arg_parser);
    let Some(app_param_name) = attrs.app_param_name else {
        return Error::new(
            attrs.app_param_name.span(),
            "auto_plugin requires attribute specifying the name of the `&mut bevy::app::App` parameter. Example: #[auto_plugin(app=app)]",
        )
            .into_compile_error()
            .into();
    };

    // Parse the input function
    let input = parse_macro_input!(input as ItemFn);
    let _func_name = &input.sig.ident;
    let func_body = &input.block;
    let func_sig = &input.sig;
    let func_vis = &input.vis;
    let func_attrs = &input.attrs;

    // TODO: tuple struct with &'static string and app_param_name ?
    let app_param_mut_check_result = is_fn_param_mutable_reference(&input, &app_param_name, FnParamMutabilityCheckErrMessages {
        not_mutable_message: "auto_plugin attribute must be used on a function with a `&mut bevy::app::App` parameter".to_string(),
        not_found_message: format!("auto_plugin could not find the parameter named `{app_param_name}` in the function signature."),
    });
    if let Err(err) = app_param_mut_check_result {
        return err.into_compile_error().into();
    }

    let injected_code = match auto_plugin_inner(get_file_path(), &app_param_name) {
        Ok(code) => code,
        Err(err) => return err.to_compile_error().into(),
    };

    let expanded = quote! {
        #(#func_attrs)*
        #func_vis #func_sig {
            #injected_code
            #func_body
        }
    };

    CompilerStream::from(expanded)
}

fn auto_plugin_inner(file_path: String, app_param_name: &Ident) -> Result<MacroStream> {
    update_file_state(file_path, |file_state| {
        if file_state.plugin_registered {
            return Err(Error::new(
                Span::call_site(),
                "plugin already registered or duplicate attribute",
            ));
        }
        file_state.plugin_registered = true;
        let register_types =
            generate_register_types(app_param_name, file_state.register_types.clone().drain())?;
        let add_events = generate_add_events(app_param_name, file_state.add_events.drain())?;
        let init_resources =
            generate_init_resources(app_param_name, file_state.init_resources.drain())?;
        Ok(quote! {
            #register_types
            #add_events
            #init_resources
        })
    })
}

struct FnParamMutabilityCheckErrMessages {
    not_mutable_message: String,
    not_found_message: String,
}
fn is_fn_param_mutable_reference(
    item: &ItemFn,
    param_ident: &Ident,
    messages: FnParamMutabilityCheckErrMessages,
) -> Result<()> {
    for arg in &item.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            let Pat::Ident(pat_ident) = &*pat_type.pat else {
                continue;
            };
            if *param_ident != pat_ident.ident {
                continue;
            }
            if !is_mutable_reference(&pat_type.ty) {
                return Err(Error::new(pat_type.span(), messages.not_mutable_message));
            }
            return Ok(());
        }
    }
    Err(Error::new(
        item.sig.inputs.span(),
        messages.not_found_message,
    ))
}

/// Check if the type is `&mut _`
fn is_mutable_reference(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Reference(TypeReference {
            mutability: Some(_),
            ..
        })
    )
}

fn update_file_state<R>(file_path: String, update_fn: impl FnOnce(&mut FileState) -> R) -> R {
    FILE_STATE_MAP.with(|map| {
        let mut map = map.borrow_mut();
        let file_state = map.entry(file_path).or_default();
        update_fn(file_state)
    })
}

fn validate_generic_counts(generics: &Generics, path: &Path) -> Result<()> {
    let expected_generics_count = generics.type_params().count();
    if expected_generics_count > 0 {
        let paths_count = count_generics(path);
        if paths_count != expected_generics_count {
            return Err(Error::new(
                path.span(),
                format!(
                    "expected {expected_generics_count} generic parameters, found {paths_count}"
                ),
            ));
        }
    }
    Ok(())
}

fn ident_to_path(ident: &Ident) -> Path {
    Path {
        leading_colon: None,
        segments: {
            let mut segments = Punctuated::new();
            segments.push(PathSegment {
                ident: ident.clone(),
                arguments: PathArguments::None,
            });
            segments
        },
    }
}

fn count_generics(path: &Path) -> usize {
    // Iterate through the segments of the path
    for segment in &path.segments {
        // Check if the segment has angle-bracketed arguments
        if let PathArguments::AngleBracketed(angle_bracketed) = &segment.arguments {
            // Count the number of arguments inside the angle brackets
            return angle_bracketed.args.len();
        }
    }
    // If no angle-bracketed arguments are found, return 0
    0
}

struct StructOrEnumRef<'a> {
    ident: &'a Ident,
    generics: &'a Generics,
}

impl<'a> StructOrEnumRef<'a> {
    fn new(ident: &'a Ident, generics: &'a Generics) -> Self {
        Self { ident, generics }
    }
}

impl<'a> TryFrom<&'a Item> for StructOrEnumRef<'a> {
    type Error = Error;

    fn try_from(item: &'a Item) -> std::result::Result<Self, Self::Error> {
        Ok(match item {
            Item::Struct(ref struct_item) => {
                StructOrEnumRef::new(&struct_item.ident, &struct_item.generics)
            }
            Item::Enum(ref enum_item) => {
                StructOrEnumRef::new(&enum_item.ident, &enum_item.generics)
            }
            _ => return Err(Error::new(item.span(), "expected struct or enum")),
        })
    }
}

fn resolve_path_from_item_or_args(
    item: Item,
    args: Option<Punctuated<Path, Comma>>,
) -> Result<Path> {
    let struct_or_enum = StructOrEnumRef::try_from(&item)?;
    let ident = struct_or_enum.ident;
    if let Some(args) = args {
        let mut args = args.into_iter();
        // Extract the single path
        let Some(path) = args.next() else {
            return Err(Error::new(item.span(), "Attribute arguments expect a path"));
        };
        if let Some(extra_arg) = args.next() {
            return Err(Error::new(
                extra_arg.span(),
                "Attribute arguments expects a single path",
            ));
        }
        let path_ident = path
            .segments
            .get(0)
            .map(|segment| &segment.ident)
            .unwrap_or_else(|| unreachable!());
        if path_ident != ident {
            let provided_path_string = path_to_string(&path, true);
            return Err(Error::new(path.span(), format!("Attribute arguments path does not match the items ident, got: {provided_path_string}, expected: {ident} (with generics if applicable)")));
        }
        validate_generic_counts(struct_or_enum.generics, &path)?;
        Ok(path)
    } else {
        Ok(ident_to_path(ident))
    }
}

enum Target {
    RegisterTypes,
    AddEvents,
    InitResources,
}

#[derive(Error, Debug)]
enum UpdateStateError {
    #[error("duplicate attribute")]
    Duplicate,
    #[error("plugin already registered above, move plugin fn to the bottom of the file")]
    PluginAlreadyRegistered,
}

fn update_state(
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
            Target::RegisterTypes => entry.register_types.insert(path),
            Target::AddEvents => entry.add_events.insert(path),
            Target::InitResources => entry.init_resources.insert(path),
        };
        if !inserted {
            return Err(UpdateStateError::Duplicate);
        }
        Ok(())
    })
}

fn get_file_path() -> String {
    proc_macro2::Span::call_site()
        .unwrap()
        .source_file()
        .path()
        .display()
        .to_string()
}

fn path_to_string(path: &Path, strip_spaces: bool) -> String {
    let path_string = quote!(#path).to_string();
    if strip_spaces {
        path_string.replace(" ", "")
    } else {
        path_string
    }
}

fn handle_attribute_inner(
    file_path: String,
    item: Item,
    attr_span: Span,
    target: Target,
    args: Option<Punctuated<Path, Comma>>,
) -> Result<()> {
    let path = resolve_path_from_item_or_args(item, args)?;

    update_state(file_path, path, target).map_err(|err| Error::new(attr_span, err))?;

    Ok(())
}

fn handle_attribute(attr: CompilerStream, input: CompilerStream, target: Target) -> CompilerStream {
    let cloned_input = input.clone();
    let parsed_item = parse_macro_input!(input as Item);
    let args = if attr.is_empty() {
        None
    } else {
        Some(parse_macro_input!(attr with Punctuated::<Path, Token![,]>::parse_terminated))
    };

    handle_attribute_inner(
        get_file_path(),
        parsed_item,
        Span::call_site(),
        target,
        args,
    )
        .map(|_| cloned_input)
        .unwrap_or_else(|err| err.to_compile_error().into())
}

fn generate_register_types(
    app_ident: &Ident,
    items: impl Iterator<Item = String>,
) -> Result<MacroStream> {
    let register_types = items
        .map(|item| {
            let item = syn::parse_str::<Path>(&item)?;
            Ok(quote! {
                #app_ident.register_type::<#item>();
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(quote! {
        {
            // register_types
            #(#register_types)*
        }
    })
}

fn generate_add_events(
    app_ident: &Ident,
    items: impl Iterator<Item = String>,
) -> Result<MacroStream> {
    let add_events = items
        .map(|item| {
            let item = syn::parse_str::<Path>(&item)?;
            Ok(quote! {
                #app_ident.add_event::<#item>();
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(quote! {
        {
            // add_events
            #(#add_events)*
        }
    })
}

fn generate_init_resources(
    app_ident: &Ident,
    items: impl Iterator<Item = String>,
) -> Result<MacroStream> {
    let init_resources = items
        .map(|item| {
            let item = syn::parse_str::<Path>(&item)?;
            Ok(quote! {
                #app_ident.init_resource::<#item>();
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(quote! {
        {
            // init_resources
            #(#init_resources)*
        }
    })
}

#[proc_macro_attribute]
pub fn auto_register_type(attr: CompilerStream, input: CompilerStream) -> CompilerStream {
    handle_attribute(attr, input, Target::RegisterTypes)
}
#[proc_macro_attribute]
pub fn auto_add_event(attr: CompilerStream, input: CompilerStream) -> CompilerStream {
    handle_attribute(attr, input, Target::AddEvents)
}
#[proc_macro_attribute]
pub fn auto_init_resource(attr: CompilerStream, input: CompilerStream) -> CompilerStream {
    handle_attribute(attr, input, Target::InitResources)
}
