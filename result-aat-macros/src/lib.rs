#![cfg_attr(docsrs, feature(doc_cfg))]

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr, parse_macro_input};

#[proc_macro]
#[doc = include_str!("../doc/macros/iart_open_no_log.md")]
pub fn iart_open_no_log(input: TokenStream) -> TokenStream {
    let e = parse_macro_input!(input as Expr);

    let expanded = quote! {
        {
            let mut iart = #e;

            match unsafe{iart.__internal_take_data()} {
                Some(::core::result::Result::Ok(item)) => {
                    unsafe{iart.__internal_mark_handled()};
                    item
                }
                Some(::core::result::Result::Err(err)) => {
                    return unsafe{::result_aat::prelude::result_aat::__internal_rebuild_err(
                        err,
                        iart.__internal_take_log(),
                        iart.__internal_get_trans_fns(),
                        iart.__internal_take_err_item(),
                        iart.__internal_get_allocator()
                    )};
                }
                None => panic!("Iart: consumed data in iart_open_no_log"),
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
#[doc = include_str!("../doc/macros/iart_try.md")]
pub fn iart_try(input: TokenStream) -> TokenStream {
    let e = parse_macro_input!(input as Expr);

    #[cfg(not(feature = "for-nightly-try-support"))]
    let expanded = quote! {
        {
            let mut iart = #e;

            {
                iart.send_log();
                unsafe{iart.__internal_send_try_used().unwrap()};

                match unsafe{iart.__internal_take_data()} {
                    Some(::core::result::Result::Ok(item)) => {
                        unsafe{iart.__internal_mark_handled()};
                        item
                    }
                    Some(::core::result::Result::Err(err)) => {
                        return unsafe{::result_aat::prelude::result_aat::__internal_rebuild_err(
                            err,
                            iart.__internal_take_log(),
                            iart.__internal_get_trans_fns(),
                            iart.__internal_take_err_item(),
                            iart.__internal_get_allocator(),
                        )};
                    }
                    None => panic!("Iart: consumed data in iart_try"),
                }
            }
        }
    };
    #[cfg(feature = "for-nightly-try-support")]
    let expanded = quote! {
        {
            let mut iart = #e;
            iart?
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(IartErr)]
#[doc = include_str!("../doc/macros/derive_iart_err.md")]
pub fn derive_iart_err(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    #[cfg(feature = "for-nightly-allocator-api-support")]
    let body = quote! {
            fn clone_box_in<'a>(&self, alloc: ::alloc::alloc::Global) -> Box<dyn ::result_aat::prelude::IartErr<::alloc::alloc::Global> + 'a + Send + Sync, ::alloc::alloc::Global>
        where
            Self: 'a,
        {
            Box::new_in(self.clone(), alloc)
        }
    };

    #[cfg(all(not(feature = "for-nightly-allocator-api-support"), feature = "alloc"))]
    let body = quote! {
            fn clone_box(&self) -> Box<dyn ::result_aat::prelude::IartErr + Send + Sync + 'static> {
            Box::new(self.clone())
        }
    };

    #[cfg(all(
        not(feature = "alloc"),
        not(feature = "for-nightly-allocator-api-support")
    ))]
    let body = quote! {};

    let expanded = quote! {
        impl ::result_aat::prelude::IartErr for #name
        where
            Self: ::core::clone::Clone + 'static
        {
            #body
        }
    };

    expanded.into()
}
