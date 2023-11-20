/*
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */
//!
//! Derive procedural macros for the [`mem_dbg`](https://crates.io/crates/mem_dbg) crate.
//!

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput};

struct CommonDeriveInput {
    name: syn::Ident,
    generics: proc_macro2::TokenStream,
    generics_names: proc_macro2::TokenStream,
    where_clause: proc_macro2::TokenStream,
}

impl CommonDeriveInput {
    fn new(
        input: DeriveInput,
        traits_to_add: Vec<syn::Path>,
        lifetimes_to_add: Vec<syn::Lifetime>,
    ) -> Self {
        let name = input.ident;
        let mut generics = quote!();
        let mut generics_names = quote!();
        if !input.generics.params.is_empty() {
            input.generics.params.iter().for_each(|x| {
                match x {
                    syn::GenericParam::Type(t) => {
                        generics_names.extend(t.ident.to_token_stream());
                    }
                    syn::GenericParam::Lifetime(l) => {
                        generics_names.extend(l.lifetime.to_token_stream());
                    }
                    syn::GenericParam::Const(c) => {
                        generics_names.extend(c.ident.to_token_stream());
                    }
                };
                generics_names.extend(quote!(,))
            });
            input.generics.params.into_iter().for_each(|x| match x {
                syn::GenericParam::Type(t) => {
                    let mut t = t;
                    for trait_to_add in traits_to_add.iter() {
                        t.bounds.push(syn::TypeParamBound::Trait(syn::TraitBound {
                            paren_token: None,
                            modifier: syn::TraitBoundModifier::None,
                            lifetimes: None,
                            path: trait_to_add.clone(),
                        }));
                    }
                    for lifetime_to_add in lifetimes_to_add.iter() {
                        t.bounds
                            .push(syn::TypeParamBound::Lifetime(lifetime_to_add.clone()));
                    }
                    generics.extend(quote!(#t,));
                }
                x => {
                    generics.extend(x.to_token_stream());
                    generics.extend(quote!(,))
                }
            });
        }

        let where_clause = input
            .generics
            .where_clause
            .map(|x| x.to_token_stream())
            .unwrap_or(quote!(where));

        Self {
            name,
            generics,
            generics_names,
            where_clause,
        }
    }
}

#[proc_macro_derive(MemSize)]
pub fn mem_dbg_mem_size(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let CommonDeriveInput {
        name,
        generics,
        generics_names,
        where_clause,
        ..
    } = CommonDeriveInput::new(
        input.clone(),
        vec![syn::parse_quote!(mem_dbg::MemSize)],
        vec![],
    );
    let out = match input.data {
        Data::Enum(e) => {
            let mut variants = Vec::new();
            let mut variant_sizes = Vec::new();
            let mut variant_capacities = Vec::new();
            e.variants.iter().for_each(|variant| {
                let mut res = variant.ident.to_owned().to_token_stream();
                let mut var_args_size = quote! {core::mem::size_of::<Self>()};
                let mut var_args_cap = quote! {core::mem::size_of::<Self>()};
                match &variant.fields {
                    syn::Fields::Unit => {}
                    syn::Fields::Named(fields) => {
                        let mut args = proc_macro2::TokenStream::new();
                        fields
                            .named
                            .iter()
                            .map(|named| {
                                (named.ident.as_ref().unwrap(), named.ty.to_token_stream())
                            })
                            .for_each(|(ident, ty)| {
                                var_args_size.extend([quote! {
                                    + #ident.mem_size() - core::mem::size_of::<#ty>()
                                }]);
                                var_args_cap.extend([quote! {
                                    + #ident.mem_capacity() - core::mem::size_of::<#ty>()
                                }]);
                                args.extend([ident.to_token_stream()]);
                                args.extend([quote! {,}]);
                            });
                        // extend res with the args sourrounded by curly braces
                        res.extend(quote! {
                            { #args }
                        });
                    }
                    syn::Fields::Unnamed(fields) => {
                        let mut args = proc_macro2::TokenStream::new();
                        fields.unnamed.iter().enumerate().for_each(|(idx, value)| {
                            let ident = syn::Ident::new(
                                &format!("v{}", idx),
                                proc_macro2::Span::call_site(),
                            )
                            .to_token_stream();
                            let ty = value.ty.to_token_stream();
                            var_args_size.extend([quote! {
                                + #ident.mem_size() - core::mem::size_of::<#ty>()
                            }]);
                            var_args_cap.extend([quote! {
                                + #ident.mem_capacity() - core::mem::size_of::<#ty>()
                            }]);
                            args.extend([ident]);
                            args.extend([quote! {,}]);
                        });
                        // extend res with the args sourrounded by curly braces
                        res.extend(quote! {
                            ( #args )
                        });
                    }
                }
                variants.push(res);
                variant_sizes.push(var_args_size);
                variant_capacities.push(var_args_cap);
            });

            quote! {
                #[automatically_derived]
                impl<#generics> mem_dbg::MemSize for #name<#generics_names> #where_clause{
                    fn mem_size(&self) -> usize {
                        match self {
                            #(
                               #name::#variants => #variant_sizes,
                            )*
                        }
                    }

                    fn mem_capacity(&self) -> usize {
                        match self {
                            #(
                               #name::#variants => #variant_capacities,
                            )*
                        }
                    }
                }
            }
        }
        Data::Struct(s) => {
            let mut fields_ident = vec![];
            let mut fields_ty = vec![];
            s.fields.iter().enumerate().for_each(|(field_idx, field)| {
                fields_ident.push(
                    field
                        .ident
                        .to_owned()
                        .map(|t| t.to_token_stream())
                        .unwrap_or_else(|| syn::Index::from(field_idx).to_token_stream()),
                );
                fields_ty.push(field.ty.to_token_stream());
            });

            quote! {
                #[automatically_derived]
                impl<#generics> mem_dbg::MemSize for #name<#generics_names> #where_clause{
                    fn mem_size(&self) -> usize {
                        let mut bytes = core::mem::size_of::<Self>();
                        #(bytes += self.#fields_ident.mem_size() - core::mem::size_of::<#fields_ty>();)*
                        bytes
                    }

                    fn mem_capacity(&self) -> usize {
                        let mut bytes = core::mem::size_of::<Self>();
                        #(bytes += self.#fields_ident.mem_capacity() - core::mem::size_of::<#fields_ty>();)*
                        bytes
                    }
                }
            }
        }

        _ => todo!(),
    };
    out.into()
}

#[proc_macro_derive(MemDbg)]
pub fn mem_dbg_mem_dbg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let CommonDeriveInput {
        name,
        generics,
        generics_names,
        where_clause,
        ..
    } = CommonDeriveInput::new(
        input.clone(),
        vec![syn::parse_quote!(mem_dbg::MemDbg)],
        vec![],
    );

    let out = match input.data {
        Data::Struct(s) => {
            let fields = s
                .fields
                .iter()
                .enumerate()
                .map(|(field_idx, field)| {
                    field
                        .ident
                        .to_owned()
                        .map(|t| t.to_token_stream())
                        .unwrap_or_else(|| syn::Index::from(field_idx).to_token_stream())
                })
                .collect::<Vec<_>>();

            let fields_str = s
                .fields
                .iter()
                .enumerate()
                .map(|(field_idx, field)| {
                    field
                        .ident
                        .to_owned()
                        .map(|t| t.to_string().to_token_stream())
                        .unwrap_or_else(|| field_idx.to_string().to_token_stream())
                })
                .collect::<Vec<_>>();

            quote! {
                #[automatically_derived]
                impl<#generics> mem_dbg::MemDbg for #name<#generics_names> #where_clause{
                    fn _mem_dbg_rec_on(
                        &self,
                        writer: &mut impl core::fmt::Write,
                        depth: usize,
                        max_depth: usize,
                        type_name: bool,
                        humanize: bool,
                    ) -> core::fmt::Result {
                        #(self.#fields.mem_dbg_depth_on(writer, depth + 1, max_depth, Some(#fields_str), type_name, humanize)?;)*
                        Ok(())
                    }
                }
            }
        }
        _ => todo!(),
    };
    out.into()
}
