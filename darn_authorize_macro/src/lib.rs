// authorize_macro/src/lib.rs

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use quote::spanned::Spanned;
use syn::{
    parse_macro_input, AttributeArgs, FnArg, Ident, ItemFn, Lit, Meta, NestedMeta, PathArguments,
    Type, TypePath,
};

#[proc_macro_attribute]
pub fn authorize(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args_attr = attr.clone();
    let args = parse_macro_input!(args_attr as AttributeArgs);
    let input_fn = parse_macro_input!(item as ItemFn);

    if args.len() != 2 {
        return syn::Error::new_spanned(
            proc_macro2::TokenStream::from(attr),
            "Expected two arguments: a permission enum variant and an extractor function",
        )
            .to_compile_error()
            .into();
    }
    
    
    

    let params = input_fn.sig.inputs.iter().collect::<Vec<_>>();

    // Example: Debug output of parameters (for verification during macro development)
    for param in params {
        eprintln!("Parameter: {:?}", param.into_token_stream().to_string());
        
    }

    // Return the original function unmodified
    TokenStream::from(quote! {
        #input_fn
    })
}
