//! # Functional trait
//!
//! ## Description
//!
//! A simple macro that inspired by java's functional interface.
//!
//! the macro impls a trait for [Fn], [FnMut] or [FnOnce] when the trait:
//!
//! - contains one and only one method
//! - the method has a receiver, and the receiver is `&self`, `&mut self` or `self`
//! - has no generics in the trait or the method (maybe I will add generics in the macro)
//! - has no super trait (may change in the future versions)
//! - is not unsafe
//! - have no unsafe method (may change in the future versions)
//!
//! ## Example
//!
//! ```
//! use functional_trait::functional_trait;
//!
//! #[functional_trait]
//! trait A {
//!     fn a(&self, i: i32, j: i32) -> i32;
//! }
//!
//! #[functional_trait]
//! trait B {
//!     fn b(&mut self, i: i32, j: i32) -> i32;
//! }
//!
//! #[functional_trait]
//! trait C {
//!     fn c(self, i: i32, j: i32) -> i32;
//! }
//!
//! let f = |a, b| a + b + 10;
//! dbg!(f.a(1, 2));
//!
//! let mut i = 0;
//! let mut f = |a, b| {
//!     i += 1;
//!     a + b + i
//! };
//! dbg!(f.b(1, 2));
//!
//! let s = String::new();
//! let f = |a, b| {
//!     drop(s);
//!     a + b + i
//! };
//! dbg!(f.c(1, 2));
//!
//! ```

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{ItemTrait, LifetimeParam, Type, TypeReference};

#[derive(Clone)]
enum ReceiverType {
    None,
    Ref(TypeReference),
    Mut(TypeReference),
    Owned,
}

impl std::fmt::Debug for ReceiverType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Ref(arg0) => f
                .debug_tuple("Ref")
                .field(&format_token_stream(arg0))
                .finish(),
            Self::Mut(arg0) => f
                .debug_tuple("Mut")
                .field(&format_token_stream(arg0))
                .finish(),
            Self::Owned => write!(f, "Owned"),
        }
    }
}

impl PartialEq for ReceiverType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ref(l0), Self::Ref(r0)) => format_token_stream(l0) == format_token_stream(r0),
            (Self::Mut(l0), Self::Mut(r0)) => format_token_stream(l0) == format_token_stream(r0),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

fn format_token_stream(t: &impl ToTokens) -> String {
    format!("{}", t.to_token_stream())
}

fn expend(input: ItemTrait) -> Result<TokenStream, String> {
    if input.generics.gt_token.is_some() || input.generics.lt_token.is_some() {
        Err("Generics not supported ")?
    }
    if input.unsafety.is_some() {
        Err("unsafe not supported")?
    }

    let trait_name = input.ident.clone();
    let func = {
        let item = if input.items.len() != 1 {
            Err("need exactly 1 fn")?
        } else {
            input.items[0].clone()
        };
        match item {
            syn::TraitItem::Fn(f) => f,
            _ => Err("need fn")?,
        }
    };

    let func_sig = { func.sig.clone() };

    if func_sig.unsafety.is_some() {
        Err("unsafe fn not supported")?
    }
    if func_sig.generics.type_params().next().is_some()
        || func_sig.generics.const_params().next().is_some()
    {
        Err("fn with generic types not supported")?
    }

    let func_liftimes: Vec<LifetimeParam> = {
        func_sig
            .generics
            .lifetimes()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
    };

    let func_name = func_sig.ident.clone();

    // let func_inputs = func_sig.inputs;

    let self_input: ReceiverType = {
        match func_sig.inputs.first() {
            Some(s) => match s.clone() {
                syn::FnArg::Receiver(r) => {
                    if r.colon_token.is_some() {
                        Err("must be &self, &mut self, Self")?
                    }
                    // println!("{}", r.mutability.to_token_stream());
                    if r.mutability.is_some() {
                        ReceiverType::Mut(match &*r.ty {
                            syn::Type::Reference(r) => r.clone(),
                            _ => unreachable!(),
                        })
                    } else {
                        match &*r.ty {
                            syn::Type::Path(_) => ReceiverType::Owned,
                            syn::Type::Reference(r) => ReceiverType::Ref(r.clone()),
                            _ => unreachable!(),
                        }
                    }
                }
                syn::FnArg::Typed(_) => ReceiverType::None,
            },
            None => ReceiverType::None,
        }
    };

    if self_input == ReceiverType::None {
        Err("must have a receiver")?
    }

    let func_inputs = {
        func_sig
            .inputs
            .iter()
            .enumerate()
            .filter(|(id, _v)| !(*id == 0 && self_input != ReceiverType::None))
            .map(|(_id, v)| v)
            .collect::<Vec<_>>()
    };

    let func_arg_ids: Vec<Ident> = func_inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Receiver(_) => Err("no receiver except for the first arg"),
            syn::FnArg::Typed(t) => match &*t.pat {
                syn::Pat::Ident(i) => Ok(i.ident.clone()),
                _ => unreachable!(),
            },
        })
        .collect::<Result<Vec<_>, _>>()?;

    let func_arg_tys: Vec<Type> = func_inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Receiver(_) => Err("no receiver except for the first arg"),
            syn::FnArg::Typed(t) => Ok((*t.ty).clone()),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let func_out_type: Type = match &func_sig.output {
        syn::ReturnType::Default => void_type(),
        syn::ReturnType::Type(_, b) => (**b).clone(),
    };

    // print_token_vec(&func_arg_ids);
    // print_token_vec(&func_arg_tys);
    // println!("{}", void_type().into_token_stream());

    let trait_impl = gen_impl(
        trait_name,
        func_name,
        self_input,
        func_arg_ids,
        func_arg_tys,
        func_out_type,
        func_liftimes,
    );

    let expanded = quote!(
        #trait_impl
    );
    // println!("{}", expanded);

    // abort()
    Ok(expanded)
}

// fn token_vec(vec: &Vec<impl ToTokens>) -> TokenStream {
//     let tokens = vec.iter().map(ToTokens::into_token_stream).collect();
//     tokens
// }
// fn print_token_vec(vec: &Vec<impl ToTokens>) {
//     println!("{}", token_vec(vec));
// }

fn void_type() -> Type {
    syn::parse_quote!(())
}

fn gen_impl(
    trait_name: Ident,
    func_name: Ident,
    self_input: ReceiverType,
    func_arg_ids: Vec<Ident>,
    func_arg_tys: Vec<Type>,
    func_out_type: Type,
    func_liftimes: Vec<LifetimeParam>,
) -> TokenStream {
    let fn_trait = match self_input {
        ReceiverType::None | ReceiverType::Ref(_) => quote!(std::ops::Fn),
        ReceiverType::Mut(_) => quote!(std::ops::FnMut),
        ReceiverType::Owned => quote!(std::ops::FnOnce),
    };

    let self_receiver = match self_input {
        ReceiverType::None => quote!(),
        ReceiverType::Ref(t) => {
            let liftimes = t.lifetime;
            quote!(&#liftimes self)
        }
        ReceiverType::Mut(t) => {
            let liftimes = t.lifetime;
            quote!(&#liftimes mut self)
        }
        ReceiverType::Owned => quote!(self),
    };

    let for_liftime = {
        if func_liftimes.is_empty() {
            quote!()
        } else {
            quote!(
                for<#(#func_liftimes),*>
            )
        }
    };

    let func_liftime_generics = {
        if func_liftimes.is_empty() {
            quote!()
        } else {
            quote!(<#(#func_liftimes),*>)
        }
    };

    quote::quote!(
        impl<F> #trait_name for F where
            F: #for_liftime #fn_trait(#(#func_arg_tys),*) ->#func_out_type,
            {
                fn #func_name #func_liftime_generics (#self_receiver, #(#func_arg_ids:#func_arg_tys),* ) -> #func_out_type{
                    self(#(#func_arg_ids),*)
                }
            }
    )
}

#[proc_macro_attribute]
pub fn functional_trait(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let d: syn::ItemTrait = syn::parse2(input.clone()).unwrap();
    let a: TokenStream = expend(d).unwrap().into_token_stream();
    quote!(
        #input
        #a
    )
    .into()
}

#[test]
fn a() {
    let a: TokenStream = quote!(
        trait A {
            fn aaa<'a>(&'a mut self, i: &i32, fqe: (i8, String)) -> &'a i8;
        }
    );

    let d: TokenStream = quote!(
        trait D {
            fn d<'c>(&self, b: &'c i32) -> &'c i32;
        }
    );

    let d: syn::ItemTrait = syn::parse2(d).unwrap();

    let a: TokenStream = expend(d).unwrap().into_token_stream();
    println!("{}", a);
}

fn _aa(_f: impl std::ops::Fn(i32)) {}
