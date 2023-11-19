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
    generics_names_raw: Vec<String>,
    consts_names_raw: Vec<String>,
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
        let mut generics_names_raw = vec![];
        let mut consts_names_raw = vec![];
        let mut generics_names = quote!();
        if !input.generics.params.is_empty() {
            input.generics.params.iter().for_each(|x| {
                match x {
                    syn::GenericParam::Type(t) => {
                        generics_names.extend(t.ident.to_token_stream());
                        generics_names_raw.push(t.ident.to_string());
                    }
                    syn::GenericParam::Lifetime(l) => {
                        generics_names.extend(l.lifetime.to_token_stream());
                    }
                    syn::GenericParam::Const(c) => {
                        generics_names.extend(c.ident.to_token_stream());
                        consts_names_raw.push(c.ident.to_string());
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
            generics_names_raw,
            consts_names_raw,
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
        Data::Struct(s) => {
            let fields = s
                .fields
                .iter()
                .enumerate()
                .map(|(field_idx, field)| 
                    field.ident.to_owned()
                    .map(|t| t.to_token_stream())
                    .unwrap_or_else(|| syn::Index::from(field_idx).to_token_stream())
                )
                .collect::<Vec<_>>();

            quote! {
                #[automatically_derived]
                impl<#generics> mem_dbg::MemSize for #name<#generics_names> #where_clause{
                    fn mem_size(&self) -> usize {
                        let mut bytes = 0;
                        #(bytes += self.#fields.mem_size();)*
                        bytes
                    }

                    fn mem_capacity(&self) -> usize {
                        let mut bytes = 0;
                        #(bytes += self.#fields.mem_capacity();)*
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
                .map(|(field_idx, field)| 
                    field.ident.to_owned()
                    .map(|t| t.to_token_stream())
                    .unwrap_or_else(|| syn::Index::from(field_idx).to_token_stream())
                )
                .collect::<Vec<_>>();

            let fields_str = s
                .fields
                .iter()
                .enumerate()
                .map(|(field_idx, field)| 
                    field.ident.to_owned()
                    .map(|t| t.to_string().to_token_stream())
                    .unwrap_or_else(|| field_idx.to_string().to_token_stream())
                )
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
