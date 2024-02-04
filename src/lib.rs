// use async_trait::async_trait;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{ItemTrait, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ReceiverType {
    None,
    Ref,
    Mut,
    Owned,
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
    if func_sig.generics.gt_token.is_some() || func_sig.generics.lt_token.is_some() {
        Err("generics fn not supported")?
    }

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
                        ReceiverType::Mut
                    } else {
                        match &*r.ty {
                            syn::Type::Path(_) => ReceiverType::Owned,
                            syn::Type::Reference(_) => ReceiverType::Ref,
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
) -> TokenStream {
    let fn_trait = match self_input {
        ReceiverType::None | ReceiverType::Ref => quote!(std::ops::Fn),
        ReceiverType::Mut => quote!(std::ops::FnMut),
        ReceiverType::Owned => quote!(std::ops::FnOnce),
    };

    let self_receiver = match self_input {
        ReceiverType::None => quote!(),
        ReceiverType::Ref => quote!(&self),
        ReceiverType::Mut => quote!(&mut self),
        ReceiverType::Owned => quote!(self),
    };

    quote::quote!(
        impl<F> #trait_name for F where
            F: #fn_trait(#(#func_arg_tys),*) ->#func_out_type,
            {
                fn #func_name(#self_receiver, #(#func_arg_ids:#func_arg_tys),* ) -> #func_out_type{
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
            fn aaa(&mut self, i: i32, fqe: (i8, String)) -> i8;
        }
    );
    let d: syn::ItemTrait = syn::parse2(a).unwrap();

    let a: TokenStream = expend(d).unwrap().into_token_stream();
    println!("{}", a);
}

fn _aa(_f: impl std::ops::Fn(i32)) {}
