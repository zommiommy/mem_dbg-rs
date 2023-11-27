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

/// Pre-parsed information for the derive macros.
#[allow(dead_code)]
struct CommonDeriveInput {
    /// The identifier of the struct.
    name: syn::Ident,
    /// The token stream to be used after `impl` in angle brackets. It contains
    /// the generics, lifetimes, and consts, with their trait bounds.
    generics: proc_macro2::TokenStream,
    /// A vector containing the identifiers of the generics.
    generics_name_vec: Vec<proc_macro2::TokenStream>,
    /// Same as `generics_name_vec`, but names are concatenated
    /// and separated by commans.
    generics_names: proc_macro2::TokenStream,
    /// A vector containing the name of generics types, represented as strings.
    generics_names_raw: Vec<String>,
    /// A vector containing the identifier of the constants, represented as strings.
    /// Used to include the const values into the type hash.
    //consts_names_raw: Vec<String>,
    /// The where clause.
    where_clause: proc_macro2::TokenStream,
}

impl CommonDeriveInput {
    /// Create a new `CommonDeriveInput` from a `DeriveInput`.
    /// Additionally, one can specify traits and lifetimes to
    /// be added to the generic types.
    fn new(input: DeriveInput, traits_to_add: Vec<syn::Path>) -> Self {
        let name = input.ident;
        let mut generics = quote!();
        let mut generics_names_raw = vec![];
        //let mut consts_names_raw = vec![];
        let mut generics_name_vec = vec![];
        let mut generics_names = quote!();
        if !input.generics.params.is_empty() {
            input.generics.params.into_iter().for_each(|x| {
                match x {
                    syn::GenericParam::Type(mut t) => {
                        generics_names.extend(t.ident.to_token_stream());
                        generics_names_raw.push(t.ident.to_string());

                        t.default = None;
                        for trait_to_add in traits_to_add.iter() {
                            t.bounds.push(syn::TypeParamBound::Trait(syn::TraitBound {
                                paren_token: None,
                                modifier: syn::TraitBoundModifier::None,
                                lifetimes: None,
                                path: trait_to_add.clone(),
                            }));
                        }
                        generics.extend(quote!(#t,));
                        generics_name_vec.push(t.ident.to_token_stream());
                    }
                    syn::GenericParam::Lifetime(l) => {
                        generics_names.extend(l.lifetime.to_token_stream());

                        generics.extend(quote!(#l,));
                        generics_name_vec.push(l.lifetime.to_token_stream());
                    }
                    syn::GenericParam::Const(mut c) => {
                        generics_names.extend(c.ident.to_token_stream());
                        //consts_names_raw.push(c.ident.to_string());

                        c.default = None; // remove the defaults from the const generics
                                          // otherwise we can't use them in the impl generics
                        generics.extend(quote!(#c,));
                        generics_name_vec.push(c.ident.to_token_stream());
                    }
                };
                generics_names.extend(quote!(,))
            });
        }

        // We add a where keyword in case we need to add clauses
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
            //consts_names_raw,
            generics_name_vec,
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
    } = CommonDeriveInput::new(input.clone(), vec![syn::parse_quote!(mem_dbg::MemSize)]);
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
    } = CommonDeriveInput::new(input.clone(), vec![syn::parse_quote!(mem_dbg::MemDbgImpl)]);

    let out = match input.data {
        Data::Struct(s) => {
            let code = s
                .fields
                .iter()
                .enumerate()
                .map(|(field_idx, field)| {
                    let field_ident = field
                        .ident
                        .to_owned()
                        .map(|t| t.to_token_stream())
                        .unwrap_or_else(|| syn::Index::from(field_idx).to_token_stream());

                    let fields_str = field
                        .ident
                        .to_owned()
                        .map(|t| t.to_string().to_token_stream())
                        .unwrap_or_else(|| field_idx.to_string().to_token_stream());

                    let is_last = field_idx == s.fields.len().saturating_sub(1);

                    quote!{self.#field_ident.mem_dbg_depth_on(_memdbg_writer, _memdbg_depth, _memdbg_max_depth, Some(#fields_str), #is_last, _memdbg_flags)?;}
                })
                .collect::<Vec<_>>();

            quote! {
                #[automatically_derived]
                impl<#generics> mem_dbg::MemDbgImpl for #name<#generics_names> #where_clause{
                    #[inline(always)]
                    fn _mem_dbg_rec_on(
                        &self,
                        _memdbg_writer: &mut impl core::fmt::Write,
                        _memdbg_depth: usize,
                        _memdbg_max_depth: usize,
                        _memdbg_is_last: bool,
                        _memdbg_flags: mem_dbg::Flags,
                    ) -> core::fmt::Result {
                        #(#code)*
                        Ok(())
                    }
                }
            }
        }
        Data::Enum(e) => {
            let mut variants = Vec::new();
            let mut variants_code = Vec::new();
            e.variants.iter().for_each(|variant| {
                let mut res = variant.ident.to_owned().to_token_stream();
                let mut variant_code = quote! {};
                match &variant.fields {
                    syn::Fields::Unit => {},
                    syn::Fields::Named(fields) => {
                        let mut args = proc_macro2::TokenStream::new();
                        fields
                            .named
                            .iter()
                            .enumerate()
                            .for_each(|(idx, named)| {
                                let ident = named.ident.as_ref().unwrap();
                                let field_name = format!("{}", ident);
                                let is_last = idx == fields.named.len().saturating_sub(1);
                                variant_code.extend([quote! {
                                    #ident.mem_dbg_depth_on(_memdbg_writer, _memdbg_depth, _memdbg_max_depth, Some(#field_name), #is_last, _memdbg_flags)?;
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
                        fields.unnamed.iter().enumerate().for_each(|(idx, _value)| {
                            let ident = syn::Ident::new(
                                &format!("v{}", idx),
                                proc_macro2::Span::call_site(),
                            )
                            .to_token_stream();
                            let field_name = format!("{}", idx);
                            let is_last = idx == fields.unnamed.len().saturating_sub(1);
                            variant_code.extend([quote! {
                                #ident.mem_dbg_depth_on(_memdbg_writer, _memdbg_depth, _memdbg_max_depth, Some(#field_name), #is_last, _memdbg_flags)?;
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
                let variant_name = format!("Variant: {}\n", variant.ident);
                variants_code.push(quote!{{
                    _memdbg_writer.write_str(#variant_name)?;
                    #variant_code
                }});
            });

            quote! {
                #[automatically_derived]
                impl<#generics> mem_dbg::MemDbgImpl  for #name<#generics_names> #where_clause{
                    #[inline(always)]
                    fn _mem_dbg_rec_on(
                        &self,
                        _memdbg_writer: &mut impl core::fmt::Write,
                        _memdbg_depth: usize,
                        _memdbg_max_depth: usize,
                        _memdbg_is_last: bool,
                        _memdbg_flags: mem_dbg::Flags,
                    ) -> core::fmt::Result {
                        _memdbg_writer.write_str(&" ".repeat(9))?;
                        _memdbg_writer.write_str(&"│".repeat(_memdbg_depth.saturating_sub(1)))?;
                        _memdbg_writer.write_char(if _memdbg_is_last { '╰' } else { '├' })?;
                        _memdbg_writer.write_char('╴')?;
                        match self {
                            #(
                               #name::#variants => #variants_code,
                            )*
                        }
                        Ok(())
                   }
                }
            }
        }
        _ => todo!(),
    };
    out.into()
}
