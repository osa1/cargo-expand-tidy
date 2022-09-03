use fxhash::FxHashSet;
use quote::{quote, ToTokens};

fn main() {
    let input = std::env::args().nth(1).unwrap();
    let file_content = std::fs::read_to_string(input).unwrap();
    let mut ast = syn::parse_file(&file_content).unwrap();
    process_file(&mut ast);
    println!("{}", ast.into_token_stream());
}

fn process_file(file: &mut syn::File) {
    let debug_trait_path_str = quote!(::core::fmt::Debug).to_string();

    let mut removed_item_indices: FxHashSet<usize> = Default::default();

    for (item_idx, item) in file.items.iter().enumerate() {
        if let syn::Item::Impl(impl_) = item {
            if let Some((_, trait_path, _)) = &impl_.trait_ {
                // TODO: Add all `automatically_derived` impls
                if trait_path.into_token_stream().to_string() == debug_trait_path_str {
                    removed_item_indices.insert(item_idx);
                }
            }
        }
    }

    // Drop removed items, add derive attributes
    let old_items = std::mem::take(&mut file.items);
    file.items = old_items
        .into_iter()
        .enumerate()
        .filter_map(|(item_idx, mut item)| {
            if removed_item_indices.contains(&item_idx) {
                return None;
            }

            if removed_item_indices.contains(&(item_idx + 1)) {
                match item {
                    syn::Item::Enum(ref mut enum_item) => {
                        enum_item.attrs.push(syn::parse_quote!(#[derive(Debug)]));
                    }
                    syn::Item::Struct(ref mut struct_item) => {
                        struct_item.attrs.push(syn::parse_quote!(#[derive(Debug)]));
                    }
                    _ => panic!(),
                }
            }

            Some(item)
        })
        .collect();
}
