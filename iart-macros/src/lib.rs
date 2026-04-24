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
                    return unsafe{::iart::prelude::Iart::__internal_rebuild_err(
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

    let expanded = quote! {
        {
            let mut iart = #e;

            #[cfg(not(feature = "iart/for-nightly-try-support"))]
            let res = {
                iart.send_log();
                unsafe{iart.__internal_send_try_used().unwrap()};

                match unsafe{iart.__internal_take_data()} {
                    Some(::core::result::Result::Ok(item)) => {
                        unsafe{iart.__internal_mark_handled()};
                        item
                    }
                    Some(::core::result::Result::Err(err)) => {
                        return unsafe{::iart::prelude::Iart::__internal_rebuild_err(
                            err,
                            iart.__internal_take_log(),
                            iart.__internal_get_trans_fns(),
                            iart.__internal_take_err_item(),
                            iart.__internal_get_allocator(),
                        )};
                    }
                    None => panic!("Iart: consumed data in iart_try"),
                }
            };
            #[cfg(feature = "iart/for-nightly-try-support")]
            let res = iart?;

            res
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(IartErr)]
#[doc = include_str!("../doc/macros/derive_iart_err.md")]
pub fn derive_iart_err(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl ::iart::prelude::IartErr for #name
        where
            Self: ::core::clone::Clone + 'static
        {
            #[cfg(feature = "iart/for-nightly-allocator-api-support")]
            fn clone_box_in<'a>(&self, alloc: ::alloc::alloc::Global) -> Box<dyn ::iart::prelude::IartErr<::alloc::alloc::Global> + 'a + Send + Sync, Global>
            where
                Self: 'a,
            {
                Box::new_in(self.clone(), alloc)
            }

            #[cfg(not(feature = "iart/for-nightly-allocator-api-support"))]
            fn clone_box(&self) -> Box<dyn ::iart::prelude::IartErr + Send + Sync + 'static> {
                Box::new(self.clone())
            }
        }
    };

    expanded.into()
}
