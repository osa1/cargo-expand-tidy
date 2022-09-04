use fxhash::FxHashMap;
use proc_macro2::Span;
use quote::{quote, ToTokens};

fn main() {
    let input = std::env::args().nth(1).unwrap();
    let file_content = std::fs::read_to_string(input).unwrap();
    let mut ast = syn::parse_file(&file_content).unwrap();
    process_file(&mut ast);
    println!("{}", ast.into_token_stream());
}

fn process_file(file: &mut syn::File) {
    let mut item_derives: FxHashMap<usize, Vec<syn::Path>> = Default::default();

    let old_items = std::mem::take(&mut file.items);

    let mut last_non_derive_item: usize = 0;

    // Collect automatic derives
    for (item_idx, item) in old_items.iter().enumerate() {
        if let syn::Item::Impl(impl_) = item {
            if is_automatically_derived(&impl_.attrs) {
                if let Some((_, trait_path, _)) = &impl_.trait_ {
                    item_derives
                        .entry(last_non_derive_item)
                        .or_default()
                        .push(trait_path.clone());
                }
            }
        } else {
            last_non_derive_item = item_idx;
        }
    }

    // Turn automatic derives into derive attributes, remove automatic derives and structural
    // partial eq
    let new_items: Vec<syn::Item> = old_items
        .into_iter()
        .enumerate()
        .filter_map(|(item_idx, mut item)| match item_derives.get(&item_idx) {
            Some(derives) => match item {
                syn::Item::Enum(ref mut enum_item) => {
                    enum_item.attrs.push(make_item_derive_attribute(derives));
                    Some(item)
                }
                syn::Item::Struct(ref mut struct_item) => {
                    struct_item.attrs.push(make_item_derive_attribute(derives));
                    Some(item)
                }
                _ => panic!(),
            },
            None => {
                if let syn::Item::Impl(impl_) = &item {
                    if is_automatically_derived(&impl_.attrs) {
                        return None;
                    }
                    if is_structural_partial_eq_derive(impl_) {
                        return None;
                    }
                }
                Some(item)
            }
        })
        .collect();

    file.items = new_items;
}

fn is_automatically_derived(attrs: &[syn::Attribute]) -> bool {
    let attr_str = quote!(automatically_derived).to_string();
    attrs
        .iter()
        .any(|attr| (&attr.path).into_token_stream().to_string() == attr_str)
}

fn is_structural_partial_eq_derive(item: &syn::ItemImpl) -> bool {
    if let Some((_, path, _)) = &item.trait_ {
        return path.into_token_stream().to_string()
            == quote!(::core::marker::StructuralPartialEq).to_string();
    }

    false
}

fn make_item_derive_attribute(derives: &[syn::Path]) -> syn::Attribute {
    syn::Attribute {
        pound_token: syn::token::Pound {
            spans: [Span::call_site()],
        },
        style: syn::AttrStyle::Outer,
        bracket_token: syn::token::Bracket {
            span: Span::call_site(),
        },
        path: syn::Path {
            leading_colon: None,
            segments: syn::punctuated::Punctuated::from_iter(std::iter::once::<syn::PathSegment>(
                syn::PathSegment {
                    ident: syn::Ident::new("derive", Span::call_site()),
                    arguments: Default::default(),
                },
            )),
        },
        tokens: quote!((#(#derives),*)),
    }
}
