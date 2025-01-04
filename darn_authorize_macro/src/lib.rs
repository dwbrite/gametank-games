// authorize_macro/src/lib.rs

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use quote::spanned::Spanned;
use syn::{parse_macro_input, AttributeArgs, Expr, FnArg, Ident, ItemFn, Lit, Meta, NestedMeta, PathArguments, Token, Type, TypePath};
use syn::parse::{Parse, ParseStream};

/// The struct that represents `[authorize(...)]`'s arguments.
struct AuthorizeMacroArgs {
    permission: Expr, // e.g. `SitePermissions::CreateGame`
    extractor: Expr,  // e.g. `|| { SiteRoles::default_namespace() }`
}

impl Parse for AuthorizeMacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the first expression (permission).
        let permission: Expr = input.parse()?;

        // Expect a comma.
        input.parse::<Token![,]>()?;

        // Parse the second expression (closure).
        let extractor: Expr = input.parse()?;

        Ok(Self {
            permission,
            extractor,
        })
    }
}

#[proc_macro_attribute]
pub fn authorize(attr: TokenStream, item: TokenStream) -> TokenStream {
    let AuthorizeMacroArgs {
        permission,
        extractor,
    } = parse_macro_input!(attr as AuthorizeMacroArgs);

    // Parse the function we're decorating
    let input_fn = parse_macro_input!(item as ItemFn);

    // Grab pieces of the original function
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_block = &input_fn.block;
    let fn_generics = &input_fn.sig.generics;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_attrs = &input_fn.attrs;

    // For debugging, you can look at each parameter in the decorated function
    // (the patch, the user_info, etc.) before deciding how to call the closure.
    // e.g.:
    for param in fn_inputs {
        eprintln!("Param: {:?}", quote! { #param }.to_string());
    }

    // This example just calls the closure with *no* arguments,
    // as if it were `|| { SiteRoles::default_namespace() }`.
    // If you want a closure like `|patch: &GameEntryPatch| ...`,
    // you'll need to find your `patch` param in `fn_inputs` and pass it.
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis async fn #fn_name #fn_generics(#fn_inputs) #fn_output {
            // 1) Invoke the closure to get our resource or Darn
            let resource_darn = (#extractor)();

            // 2) Check permission
            //    This is where you'd do something like:
            //    check_permission(#permission, &resource_darn, ...);

            // 3) Then run the original function body
            #fn_block
        }
    };

    TokenStream::from(expanded)
}