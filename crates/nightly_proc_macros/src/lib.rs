use proc_macro::TokenStream as CompilerStream;
use proc_macro2::TokenStream as MacroStream;

#[cfg(feature = "missing_auto_plugin_check")]
use nightly_shared::files_missing_plugin_ts;
use nightly_shared::{get_file_path, update_file_state, update_state};
use proc_macro2::{Ident, Span};
use quote::quote;
use shared::util::{resolve_path_from_item_or_args, FnParamMutabilityCheckErrMessages, Target};
use shared::{generate_add_events, generate_init_resources, generate_register_types, util};
use syn::meta::ParseNestedMeta;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{parse_macro_input, Error, Item, ItemFn, Path, Result, Token};

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
    let app_param_mut_check_result = util::is_fn_param_mutable_reference(&input, &app_param_name, FnParamMutabilityCheckErrMessages {
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

    #[cfg(feature = "missing_auto_plugin_check")]
    let injected_code = {
        let output = files_missing_plugin_ts();
        quote! {
            #output
            #injected_code
        }
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
        let register_types = generate_register_types(
            app_param_name,
            file_state.context.register_types.clone().drain(),
        )?;
        let add_events =
            generate_add_events(app_param_name, file_state.context.add_events.drain())?;
        let init_resources =
            generate_init_resources(app_param_name, file_state.context.init_resources.drain())?;
        Ok(quote! {
            #register_types
            #add_events
            #init_resources
        })
    })
}

fn handle_attribute_inner(
    file_path: String,
    item: Item,
    attr_span: Span,
    target: Target,
    args: Option<Punctuated<Path, Comma>>,
) -> Result<()> {
    let path = resolve_path_from_item_or_args(&item, args)?;

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
