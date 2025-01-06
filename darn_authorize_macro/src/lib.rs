use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, FnArg, ItemFn, Pat,
    PatType, Token,
};

struct AuthorizeMacroArgs {
    permission: Expr, // e.g. `SitePermissions::CreateGame`
    extractor: Expr,  // e.g. `|patch: &GameEntryPatch| { ... }`
}

impl Parse for AuthorizeMacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let permission: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
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

    let input_fn = parse_macro_input!(item as ItemFn);

    // Grab pieces of the original function
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_block = &input_fn.block;
    let fn_generics = &input_fn.sig.generics;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_attrs = &input_fn.attrs;

    // We’ll store references to the decorated fn’s parameters:
    // ( param_ident, param_type_string )
    let mut decorated_fn_params = vec![];

    for fn_input in fn_inputs.iter() {
        match fn_input {
            FnArg::Receiver(_) => {
                // e.g. self
                // Not typical in an Axum handler, but we can handle it if needed
            }
            FnArg::Typed(pat_ty) => {
                // The type (e.g. "Json<GameEntryPatch>")
                let param_ty_string = pat_ty.ty.to_token_stream().to_string();

                // The pattern might be something like "Json(patch)"
                match &*pat_ty.pat {
                    syn::Pat::Ident(pat_ident) => {
                        // x: SomeType
                        let param_ident = &pat_ident.ident;
                        decorated_fn_params.push((param_ident.clone(), param_ty_string));
                    }
                    syn::Pat::TupleStruct(pat_tuple_struct) => {
                        // e.g. Json(patch)
                        // The type name is in `pat_tuple_struct.path`
                        // The fields are in `pat_tuple_struct.pat.elems`

                        // Just being safe: check if there's exactly one field
                        if pat_tuple_struct.pat.elems.len() == 1 {
                            // Extract that single element
                            let first_field = pat_tuple_struct.pat.elems.first().unwrap();

                            // That might be a Pat::Ident if it's "patch"
                            if let syn::Pat::Ident(field_ident) = first_field {
                                // e.g. patch
                                let param_ident = &field_ident.ident;
                                decorated_fn_params.push((param_ident.clone(), param_ty_string));
                            } else {
                                println!("Skipping complex field pattern: {}",
                                         quote! { #first_field }.to_string());
                            }
                        } else {
                            println!("Skipping multi-field pattern: {}",
                                     pat_ty.to_token_stream());
                        }
                    }
                    _ => {
                        println!("Skipping unsupported pattern: {}", pat_ty.to_token_stream());
                    }
                }
            }
        }
    }

    // Now, parse the extractor expression and see if it's a closure
    let closure = match extractor {
        Expr::Closure(expr_closure) => expr_closure,
        _ => {
            return syn::Error::new_spanned(
                extractor,
                "Expected a closure expression for the second argument, e.g. `|patch: &GameEntryPatch| { ... }`",
            )
                .to_compile_error()
                .into();
        }
    };

    // The closure param list is typically a `Punctuated<Pat, Token![,]>`
    // We might attempt to figure out the types from `PatType` nodes in the closure’s inputs.
    let closure_params: Vec<_> = closure.inputs.clone().into_iter().collect();

    // We'll create a vector of argument identifiers to pass to the closure call.
    let mut closure_call_args = Vec::new();

    for closure_param in closure_params.iter() {
        match closure_param {
            // e.g. `|patch: &GameEntryPatch|`
            // here param is like `PatType { pat, ty, ... }`
            Pat::Type(PatType { pat: _, ty, .. }) => {
                // closure param name is e.g. `patch`, but we might not strictly need it here
                // since it’s within the closure. We do need to figure out the type though.
                let closure_type_str = strip_reference_if_present(ty).to_string();

                // Attempt to find a matching param from the decorated fn by naive substring
                // e.g. if closure_type_str contains "GameEntryPatch", we want the param whose type is "Json<GameEntryPatch>" or something.
                // This is a hack—syn matching is tricky. YMMV.
                let maybe_match = decorated_fn_params.iter().find(|(_, fn_type)| {
                    // naive approach: if the function param type string
                    // contains the closure param type string, we call that "good enough"
                    fn_type.contains(&closure_type_str)
                });


                match maybe_match {
                    Some((param_ident, _)) => {
                        // We'll pass `&param_ident` to the closure
                        closure_call_args.push(quote! { &#param_ident });
                    }
                    None => {
                        // We didn't find a param. We'll fail for now
                        return syn::Error::new_spanned(
                            ty,
                            format!("Could not find a matching param in the decorated function for closure type `{:?}`", closure_type_str)
                        )
                            .to_compile_error()
                            .into();
                    }
                }
            }

            // If the closure param is just a pattern with no explicit type,
            // e.g. `|patch| { ... }`, we don't know what type it is.
            // You *could* do something more elaborate, or just bail.
            Pat::Ident(pat_ident) => {
                // No type given. We can try to guess or just bail out.
                return syn::Error::new_spanned(
                    pat_ident,
                    "Closure parameters must be typed, e.g. `|patch: &GameEntryPatch| { ... }`",
                )
                    .to_compile_error()
                    .into();
            }
            _ => {
                // Some other pattern we haven't handled
                return syn::Error::new_spanned(
                    closure_param,
                    "Unsupported closure parameter pattern",
                )
                    .to_compile_error()
                    .into();
            }
        }
    }

    // Now we have a vector of arguments to pass to the closure (e.g. [&patch, &user_info])
    // Let's reconstruct the closure body as an expression we can call.
    // The closure itself is something like `|patch: &GameEntryPatch| { Darn::with_namespace(..., patch.game_id) }`
    // We'll call it like `(closure)(...args...)`.

    // We'll also preserve the closure’s body. The closure expression includes both
    // the parameters and the body, so we can just treat it as a function object.
    // Something like: `( |patch: &GameEntryPatch| { ... } )(&patch)`
    let call_extractor = quote! {
        (#closure)(#(#closure_call_args),*)
    };


    // Put it all together in a new function
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis async fn #fn_name #fn_generics(#fn_inputs) #fn_output {
            // Here you'd presumably have access to your app_state, user_info, etc.
            // to do the actual permission check. For simplicity, let's assume
            // `app_state` and `user_info` are in scope. If they're parameters of
            // the function, you'd extract them similarly to how we matched `patch`.
            //
            // 1) Build the resource using the closure
            let resource_darn = #call_extractor;

            // 2) Check permission
            //    This snippet is just an example:
            app_state.casbin.enforce_http(&user_info, #permission, &resource_darn).await?;

            // 3) Then run the original function body
            #fn_block
        }
    };

    TokenStream::from(expanded)
}

fn strip_reference_if_present(ty: &syn::Type) -> proc_macro2::TokenStream {
    match ty {
        syn::Type::Reference(type_ref) => {
            // This is the underlying type, e.g. `GameEntryPatch`
            let elem = &type_ref.elem;
            quote::quote! { #elem }
        }
        _ => {
            quote::quote! { #ty }
        }
    }
}

