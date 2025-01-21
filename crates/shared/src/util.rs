use proc_macro2::Ident;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{
    Attribute, Error, FnArg, Generics, Item, ItemFn, ItemMod, Pat, Path, PathArguments,
    PathSegment, Token, Type, TypeReference,
};

pub fn resolve_path_from_item_or_args(
    item: &Item,
    args: Option<Punctuated<Path, Comma>>,
) -> syn::Result<Path> {
    let struct_or_enum = StructOrEnumRef::try_from(item)?;
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

pub fn path_to_string(path: &Path, strip_spaces: bool) -> String {
    let path_string = quote!(#path).to_string();
    if strip_spaces {
        path_string.replace(" ", "")
    } else {
        path_string
    }
}

pub enum Target {
    RegisterTypes,
    AddEvents,
    InitResources,
}

pub struct StructOrEnumRef<'a> {
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub attributes: &'a Vec<Attribute>,
}

impl<'a> StructOrEnumRef<'a> {
    fn new(ident: &'a Ident, generics: &'a Generics, attributes: &'a Vec<Attribute>) -> Self {
        Self {
            ident,
            generics,
            attributes,
        }
    }
}

impl<'a> TryFrom<&'a Item> for StructOrEnumRef<'a> {
    type Error = Error;

    fn try_from(item: &'a Item) -> std::result::Result<Self, Self::Error> {
        Ok(match item {
            Item::Struct(ref struct_item) => StructOrEnumRef::new(
                &struct_item.ident,
                &struct_item.generics,
                &struct_item.attrs,
            ),
            Item::Enum(ref enum_item) => {
                StructOrEnumRef::new(&enum_item.ident, &enum_item.generics, &enum_item.attrs)
            }
            _ => return Err(Error::new(item.span(), "expected struct or enum")),
        })
    }
}

pub struct FnParamMutabilityCheckErrMessages {
    pub not_mutable_message: String,
    pub not_found_message: String,
}

pub fn is_fn_param_mutable_reference(
    item: &ItemFn,
    param_ident: &Ident,
    messages: FnParamMutabilityCheckErrMessages,
) -> syn::Result<()> {
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
pub fn is_mutable_reference(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Reference(TypeReference {
            mutability: Some(_),
            ..
        })
    )
}

pub fn validate_generic_counts(generics: &Generics, path: &Path) -> syn::Result<()> {
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

pub fn ident_to_path(ident: &Ident) -> Path {
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

pub fn count_generics(path: &Path) -> usize {
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

pub fn get_all_items_in_module_by_attribute(
    module: &ItemMod,
    attribute_name: &'static str,
) -> syn::Result<Vec<ItemWithAttributeMatch>> {
    let Some((_, items)) = &module.content else {
        return Ok(vec![]);
    };

    // Find all items with the provided [`attribute_name`] #[...] attribute
    let matched_items = items_with_attribute_macro(&items, attribute_name)?;
    Ok(matched_items)
}

pub fn inject_module(
    module: &mut ItemMod,
    func: impl FnOnce() -> syn::Result<syn::Item>,
) -> syn::Result<()> {
    // Combine the original module with the generated code
    if let Some((_brace, items)) = module.content.as_mut() {
        // Inject code into the module block
        items.push(func()?);
    }

    Ok(())
}

pub struct ItemWithAttributeMatch {
    pub item: Item,
    pub path: Path,
    pub attributes: Attribute,
}

impl ItemWithAttributeMatch {
    pub fn into_path_string(self) -> String {
        path_to_string(&self.path, false)
    }
}

pub fn items_with_attribute_macro(
    items: &Vec<syn::Item>,
    attribute_name: &'static str,
) -> syn::Result<Vec<ItemWithAttributeMatch>> {
    let is_marker = |attr: &&Attribute| -> bool { attr.path().is_ident(attribute_name) };

    fn parse(ident: &Ident, attr: &Attribute) -> syn::Result<syn::Path> {
        let mut has_args = false;
        let _ = attr.parse_nested_meta(|_| {
            has_args = true;
            Ok(())
        });
        if has_args {
            let paths =
                attr.parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated)?;

            // Ensure exactly one path is present
            if paths.len() == 1 {
                // Extract the single path
                let path = paths.into_iter().next().unwrap_or_else(|| unreachable!());
                let path_ident = path
                    .segments
                    .get(0)
                    .map(|segment| &segment.ident)
                    .unwrap_or_else(|| unreachable!());
                if path_ident == ident {
                    Ok(path)
                } else {
                    let provided_path_string = quote!(#path).to_string().replace(" ", "");
                    Err(syn::Error::new(path.span(), format!("Attribute arguments path does not match the items ident, got: {provided_path_string}, expected: {ident} (with generics if applicable)")))
                }
            } else {
                Err(syn::Error::new(
                    attr.span(),
                    "Attribute arguments expect exactly one path",
                ))
            }
        } else {
            // allow #[attribute] without args
            if let Some(segment) = attr.path().segments.last() {
                if !segment.arguments.is_empty() {
                    // this should be unreachable from testing
                    Err(syn::Error::new(
                        attr.span(),
                        "Unexpected arguments (bad proc macro logic)",
                    ))
                } else {
                    Ok(ident_to_path(ident))
                }
            } else {
                Err(syn::Error::new(
                    attr.span(),
                    "Attribute arguments expect exactly one path",
                ))
            }
        }
    }

    let mut matched_items = vec![];
    for item in items {
        let Ok(matched_item) = StructOrEnumRef::try_from(item) else { 
            continue;
        };
        for attr in matched_item.attributes.iter().filter(is_marker) {
            let path = parse(matched_item.ident, attr)?;
            validate_generic_counts(matched_item.generics, &path)?;
            matched_items.push(ItemWithAttributeMatch { item: item.clone(), path, attributes: attr.clone() });
        }
    }
    Ok(matched_items)
}
