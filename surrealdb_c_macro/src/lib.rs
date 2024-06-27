extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro]
pub fn my_macro(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::LitStr);
    let value = input.value();
    let expanded = quote! {
        fn hello_macro() {
            println!("Hello, {}!", #value);
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn with_surreal(code: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(code as syn::Item);
    let out = quote! {
        {
            // let surreal = unsafe { Box::from_raw(db) };

            // let fut = #input;

            // let res = surreal.rt.block_on(fut.into_future()).unwrap();
            // let ver_str = CString::new(res.to_string()).unwrap().into_raw();

            // Box::leak(surreal);
            println!(foo);
        }
    };

    todo!()
}
