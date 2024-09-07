#![doc = include_str!("../readme.md")]

use proc_macro2::{Ident, Span, TokenStream};
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
    // if input.generics.gt_token.is_some() || input.generics.lt_token.is_some() {
    //     Err("Generics not supported ")?
    // }
    let trait_generics: Vec<syn::GenericParam> =
        input.generics.params.iter().cloned().collect::<Vec<_>>();
    // println!("{}", quote!(#(#generics),*));
    let trait_where: Vec<syn::WherePredicate> = input
        .generics
        .where_clause
        .map(|w| w.predicates.iter().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    if input.unsafety.is_some() {
        Err("unsafe trait not supported")?
    }
    let supertraits: Vec<syn::TypeParamBound> =
        input.supertraits.iter().cloned().collect::<Vec<_>>();
    // println!("{}", quote!(#(#supertraits),*));
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

    let func_is_unsafe = func_sig.unsafety.is_some();
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
        func_is_unsafe,
        supertraits,
        trait_generics,
        trait_where,
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
#[allow(clippy::too_many_arguments)]
fn gen_impl(
    trait_name: Ident,
    func_name: Ident,
    self_input: ReceiverType,
    func_arg_ids: Vec<Ident>,
    func_arg_tys: Vec<Type>,
    func_out_type: Type,
    func_liftimes: Vec<LifetimeParam>,
    func_is_unsafe: bool,
    supertraits: Vec<syn::TypeParamBound>,
    trait_generics: Vec<syn::GenericParam>,
    trait_where: Vec<syn::WherePredicate>,
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

    let func_is_unsafe = {
        if func_is_unsafe {
            quote!(unsafe)
        } else {
            quote!()
        }
    };
    let supertraits = {
        if supertraits.is_empty() {
            quote!()
        } else {
            quote!(: #(#supertraits)+*)
        }
    };

    let trait_generics_generics = {
        if trait_generics.is_empty() {
            quote!()
        } else {
            quote!(#(#trait_generics,)*)
        }
    };
    let trait_generics_trait = {
        if trait_generics.is_empty() {
            quote!()
        } else {
            // let a = quote::quote! {'a};
            // let a = a.into_iter().next().unwrap();
            let t = trait_generics.iter().map(|p| match p {
                syn::GenericParam::Lifetime(lt) => {
                    let lt = &lt.lifetime;
                    quote! {#lt}
                }
                syn::GenericParam::Type(ty) => {
                    let ty = &ty.ident;
                    quote! {#ty}
                }
                syn::GenericParam::Const(co) => {
                    let co = &co.ident;
                    quote! { #co}
                }
            });
            quote!(<#(#t),*>)
        }
    };

    let trait_where = { quote!(#(#trait_where),*) };
    let func_generic_name = Ident::new(
        "FPleaseDontUsThisIdent1193r797g31r7jh930hc931rg",
        Span::call_site(),
    );

    quote::quote!(
        #[allow(non_camel_case_types)]
        impl<#trait_generics_generics #func_generic_name #supertraits> #trait_name #trait_generics_trait for #func_generic_name where
            #func_generic_name: #for_liftime #fn_trait(#(#func_arg_tys),*) ->#func_out_type,
            #trait_where
            {
                #func_is_unsafe fn #func_name #func_liftime_generics (#self_receiver, #(#func_arg_ids:#func_arg_tys),* ) -> #func_out_type{
                    self(#(#func_arg_ids),*)
                }
            }
    )
}

///
///
/// A simple macro that inspired by java's functional interface.
///
/// the macro impls a trait for [Fn], [FnMut] or [FnOnce] when the trait:
///
/// - contains one and only one method
///
/// - the method has a receiver, and the receiver is `&self`, `&mut self` or `self`
///
/// - has no generic types in the method
///
/// - is not unsafe
///
/// # Example
///
/// ```
/// use functional_trait::functional_trait;
///
/// #[functional_trait]
/// trait E<'a, T: 'a + ?Sized, const AA: usize, T1>: Sized + Clone + Send
/// where
///     T1: Send + Sync,
///     T: std::fmt::Display,
/// {
///     unsafe fn e<'c>(&'c self, a: &'a T, b: [i32; AA], t1: T1) -> &'a str;
/// }
///
/// let fe = |a: &str, b: [i32; 4], _c: i128| {
///     dbg!(a);
///     dbg!(b);
///     "413"
/// };
/// unsafe { fe.e("4fr13", [3, 5, 1, 1], 9) };
///
/// ```
///
///
#[proc_macro_attribute]
pub fn functional_trait(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let a = || -> Result<proc_macro::TokenStream, String> {
        let input: TokenStream = input.into();
        let d: syn::ItemTrait = syn::parse2(input.clone()).map_err(|e| e.to_string())?;
        let a: TokenStream = expend(d)?.into_token_stream();
        Ok(quote!(
            #input
            #a
        )
        .into())
    };
    match a() {
        Ok(v) => v,
        Err(e) => quote! {compile_error!(#e);}.into(),
    }
}

#[test]
fn a() {
    let _a: TokenStream = quote!(
        trait A {
            fn aaa<'a>(&'a mut self, i: &i32, fqe: (i8, String)) -> &'a i8;
        }
    );

    let _b: TokenStream = quote!(
        trait D {
            fn d<'c>(&self, b: &'c i32) -> &'c i32;
        }
    );

    let d: TokenStream = quote!(
        trait D<'a, T: Sized>: Send + Sync
        where
            T: Send,
        {
            fn d<'c>(&self, b: &'c i32) -> &'c i32;
        }
    );

    let d: syn::ItemTrait = syn::parse2(d).unwrap();

    let a: TokenStream = expend(d).unwrap().into_token_stream();
    println!("{}", a);
    let e = "ffff";
    println!("{}", quote! {compile_error!(#e);});
}

fn _aa(_f: impl std::ops::Fn(i32)) {}
