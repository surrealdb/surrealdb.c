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

#[proc_macro]
pub fn def_result(name: TokenStream) -> TokenStream {
    let path = syn::parse_macro_input!(name as syn::Path);
    let last_name = &path.segments.last().unwrap().ident;
    let path_str = stringify!(path.clone());

    let name = format!("{last_name}Result");
    format!(
        r#"
#[repr(C)]
pub struct {name} {{
    ok: *mut {path_str},
    err: *mut c_char,
}}
"#
    )
    .parse()
    .unwrap()
}
