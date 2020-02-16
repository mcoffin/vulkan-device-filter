#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;
extern crate inflector;

use proc_macro::TokenStream;
use syn::{
    parse::{
        Parse,
        ParseStream,
    },
    parse_macro_input,
    Token,
};
use quote::quote;
use inflector::Inflector;

#[allow(dead_code)]
struct AttributeInput {
    real_name: syn::Ident,
    separator: Token![,],
    repr_type: syn::Ident,
}

impl AttributeInput {
    fn builder_ident(&self) -> syn::Ident {
        let builder_ident_s = format!("{}Builder", &self.real_name);
        syn::Ident::new(builder_ident_s.as_str(), self.real_name.span())
    }
}

impl Parse for AttributeInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(AttributeInput {
            real_name: input.parse()?,
            separator: input.parse()?,
            repr_type: input.parse()?,
        })
    }
}

#[proc_macro_attribute]
pub fn flag_builder(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attribute as AttributeInput);
    let enum_input = parse_macro_input!(item as syn::ItemEnum);
    let variant_functions: Vec<_> = enum_input.variants.iter()
        .map(|variant| {
            let variant_ident_s = format!("{}", &variant.ident);
            let snake = variant_ident_s.to_snake_case();
            let get_ident = syn::Ident::new(snake.as_str(), variant.ident.span());
            let set_ident = syn::Ident::new(format!("set_{}", &snake).as_str(), variant.ident.span());
            let enum_ident = &enum_input.ident.clone();
            let variant_ident = &variant.ident.clone();
            let repr_type_ident = &args.repr_type;
            let builder_ident = &args.builder_ident();
            let real_fns = quote! {
                pub fn #get_ident(&self) -> bool {
                    (self.0 & (#enum_ident::#variant_ident as #repr_type_ident)) != 0
                }

                pub fn #set_ident(&mut self) {
                    self.0 |= #enum_ident::#variant_ident as #repr_type_ident;
                }
            };
            let get_ident = get_ident.clone();
            let builder_fns = quote! {
                pub fn #get_ident(self, v: bool) -> Self {
                    if v {
                        let mask = #enum_ident::#variant_ident as #repr_type_ident;
                        #builder_ident(self.0 | mask)
                    } else {
                        let mask = (#enum_ident::#variant_ident as #repr_type_ident) ^ #repr_type_ident::max_value();
                        #builder_ident(self.0 & mask)
                    }
                }
            };
            (real_fns, builder_fns)
        })
        .collect();
    let real_ident = &args.real_name;
    let repr_type = &args.repr_type;
    let builder_ident = &args.builder_ident();
    let real_fns = variant_functions.iter()
        .map(|&(ref real, ref _builder)| real);
    let builder_fns = variant_functions.iter()
        .map(|&(ref _real, ref builder)| builder);
    TokenStream::from(quote! {
        #enum_input

        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct #real_ident(#repr_type);

        impl #real_ident {
            #(#real_fns)*
        }

        impl Into<#repr_type> for #real_ident {
            #[inline]
            fn into(self) -> #repr_type {
                self.0
            }
        }

        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct #builder_ident(#repr_type);

        impl #builder_ident {
            #(#builder_fns)*
        }

        impl Into<#real_ident> for #builder_ident {
            #[inline]
            fn into(self) -> #real_ident {
                #real_ident(self.0)
            }
        }

        impl From<#real_ident> for #builder_ident {
            #[inline]
            fn from(real: #real_ident) -> Self {
                #builder_ident(real.0)
            }
        }

        impl Into<#repr_type> for #builder_ident {
            fn into(self) -> #repr_type {
                let v: #real_ident = self.into();
                v.into()
            }
        }
    })
}
