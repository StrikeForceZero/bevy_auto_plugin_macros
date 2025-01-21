use proc_macro::TokenStream as CompilerStream;
use proc_macro2::TokenStream as MacroStream;

use proc_macro2::{Ident, Span};
use quote::quote;
use shared::util::{inject_module, items_with_attribute_macro, ItemWithAttributeMatch};
use shared::{generate_add_events, generate_init_resources, generate_register_types};
use syn::meta::ParseNestedMeta;
use syn::{parse2, parse_macro_input, Item, ItemMod, Result};

#[derive(Default)]
struct AutoPluginAttributes {
    init_name: Option<Ident>,
}

impl AutoPluginAttributes {
    fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        if meta.path.is_ident("init_name") {
            self.init_name = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("unsupported attribute"))
        }
    }
    fn init_name(&self) -> Ident {
        self.init_name
            .as_ref()
            .cloned()
            .unwrap_or(Ident::new("init", Span::call_site()))
    }
}

#[proc_macro_attribute]
pub fn auto_plugin(attr: CompilerStream, input: CompilerStream) -> CompilerStream {
    let mut attrs = AutoPluginAttributes::default();
    let arg_parser = syn::meta::parser(|meta| attrs.parse(meta));
    parse_macro_input!(attr with arg_parser);

    // Parse the input module
    let module = parse_macro_input!(input as ItemMod);

    let injected_module = match auto_plugin_inner(module, &attrs.init_name()) {
        Ok(code) => code,
        Err(err) => return err.to_compile_error().into(),
    };

    CompilerStream::from(injected_module)
}

fn auto_plugin_inner(mut module: ItemMod, init_name: &Ident) -> Result<MacroStream> {
    let app_param_ident = Ident::new("app", Span::call_site());
    // Extract the content inside the module
    if let Some((_, items)) = &module.content {
        fn map_to_string(
            iter: impl IntoIterator<Item = ItemWithAttributeMatch>,
        ) -> impl Iterator<Item = String> {
            iter.into_iter()
                .map(ItemWithAttributeMatch::into_path_string)
        }

        // Find all items with the provided [`attribute_name`] #[...] attribute
        let auto_register_types = items_with_attribute_macro(items, "auto_register_type")?;
        let auto_register_types = map_to_string(auto_register_types);

        let auto_add_events = items_with_attribute_macro(items, "auto_add_event")?;
        let auto_add_events = map_to_string(auto_add_events);

        let auto_init_resources = items_with_attribute_macro(items, "auto_init_resource")?;
        let auto_init_resources = map_to_string(auto_init_resources);

        inject_module(&mut module, move || {
            let auto_register_types =
                generate_register_types(&app_param_ident, auto_register_types)?;
            let auto_add_events = generate_add_events(&app_param_ident, auto_add_events)?;
            let auto_init_resources =
                generate_init_resources(&app_param_ident, auto_init_resources)?;
            parse2::<Item>(quote! {
                pub(super) fn #init_name(app: &mut bevy_app::prelude::App) {
                    #auto_register_types
                    #auto_add_events
                    #auto_init_resources
                }
            })
        })?;
    }

    let output = quote! {
        #module
    };

    Ok(output)
}

#[proc_macro_attribute]
pub fn auto_register_type(_args: CompilerStream, input: CompilerStream) -> CompilerStream {
    // Just return the input unchanged; this acts as a marker.
    input
}
#[proc_macro_attribute]
pub fn auto_add_event(_args: CompilerStream, input: CompilerStream) -> CompilerStream {
    // Just return the input unchanged; this acts as a marker.
    input
}
#[proc_macro_attribute]
pub fn auto_init_resource(_args: CompilerStream, input: CompilerStream) -> CompilerStream {
    // Just return the input unchanged; this acts as a marker.
    input
}
