#![feature(let_chains)]
use proc_macro::TokenStream;
use quote::{quote, ToTokens, format_ident};
use syn::{parse_macro_input, FnArg, ItemFn, PathArguments, ReturnType, Type, parse_quote};

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);
    if let ReturnType::Type(_, output) = &function.sig.output
        && let Type::Path(output) = output.as_ref()
        && output.to_token_stream().to_string().starts_with("Result")
    {
        let name = function.sig.ident;
        let vis = function.vis;
        let inputs = &mut function.sig.inputs;
        if inputs.is_empty() {
            inputs.insert(0, parse_quote!(app: tauri::AppHandle))
        };
        let inputs = &function.sig.inputs;
        let input_idents = function.sig.inputs.iter().filter_map(|input| {
            if let FnArg::Typed(input) = input {
                Some(&input.pat)
            } else {
                None
            }
        }).collect::<Vec<_>>();
        let asyncness = function.sig.asyncness;
        let await_token = asyncness.map(|_| quote!(.await));
        let output_ok = output.path.segments.last().and_then(|s| {
            if let PathArguments::AngleBracketed(a) = &s.arguments {
                a.args.first()
            } else {
                None
            }
        });
        let app = input_idents.first();
        let app_clone = app.map(|id| format_ident!("{}_clone", id.to_token_stream().to_string()));
        let body = &function.block;
        quote!(#[tauri::command]
            #vis #asyncness fn #name(#inputs) -> Result<#output_ok, std::string::String> {
                let #app_clone = #app.app_handle();
                (#asyncness move |#inputs| -> anyhow::Result<#output_ok> #body)(#(#input_idents),*)#await_token.map_err(|err| {
                    crate::app_handle_ext::AppHandleExt::log(&#app_clone, &err, crate::log::LogLevel::Error);
                    err.to_string()
                })
            }
        )
        .into()
    } else {
        quote!(#[tauri::command]
            #function)
        .into()
    }
}
