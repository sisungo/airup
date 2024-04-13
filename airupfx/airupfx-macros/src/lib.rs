use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{FnArg, ItemFn, ReturnType};

#[proc_macro_attribute]
pub fn api(_: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: ItemFn = match syn::parse2(item.clone().into()) {
        Ok(it) => it,
        Err(e) => return token_stream_with_error(item.into(), e).into(),
    };

    let mut tuple_type = TokenStream::new();
    tuple_type.append_separated(
        input.sig.inputs.iter().filter_map(|x| match x {
            FnArg::Receiver(_) => None,
            FnArg::Typed(y) => Some(y.ty.clone()),
        }),
        quote!(,),
    );

    let mut pat_args = TokenStream::new();
    pat_args.append_separated(
        input.sig.inputs.iter().filter_map(|x| match x {
            FnArg::Receiver(_) => None,
            FnArg::Typed(y) => Some(y.pat.clone()),
        }),
        quote!(,),
    );

    let ident = input.sig.ident;
    let body = input.block;
    let vis = input.vis;
    let args = input.sig.inputs;
    let asyncness = input.sig.asyncness;
    let ret = match input.sig.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => quote!(#ty),
    };

    quote! {
        #vis fn #ident(req: ::airup_sdk::ipc::Request) -> MethodFuture {
            #asyncness fn _airupfx_macro_internal_fn(#args) -> #ret #body
            Box::pin(async move {
                let (#pat_args): (#tuple_type) = req.extract_params()?;
                _airupfx_macro_internal_fn(#pat_args)
                    .await
                    .map(|x| {
                        ::ciborium::Value::serialized(&x).expect("IPC methods should return a value that can be serialized into CBOR")
                    })
            })
        }
    }
    .into()
}

fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> TokenStream {
    tokens.extend(error.into_compile_error());
    tokens
}
