/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Derive procedural macros for the [`mem_dbg`](https://crates.io/crates/mem_dbg) crate.

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, parse_quote, parse_quote_spanned, spanned::Spanned, Data, DeriveInput,
};

/**

Generate a `mem_dbg::MemSize` implementation for custom types.

Presently we do not support unions.

The attribute `copy_type` can be used on [`Copy`] types that do not contain references
to make `MemSize::mem_size` faster on arrays, vectors and slices. Note that specifying
`copy_type` will add the bound that the type is `Copy + 'static`.

See `mem_dbg::CopyType` for more details.

*/
#[proc_macro_derive(MemSize, attributes(copy_type))]
pub fn mem_dbg_mem_size(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    input.generics.make_where_clause();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.unwrap().clone(); // We just created it

    let is_copy_type = input
        .attrs
        .iter()
        .any(|x| x.meta.path().is_ident("copy_type"));

    let copy_type: syn::Expr = if is_copy_type {
        where_clause
            .predicates
            .push(parse_quote_spanned!(name.span()=> Self: Copy + 'static));
        parse_quote!(mem_dbg::True)
    } else {
        parse_quote!(mem_dbg::False)
    };

    match input.data {
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
                let ty = &field.ty;
                where_clause
                    .predicates
                    .push(parse_quote_spanned!(field.span()=> #ty: mem_dbg::MemSize));
            });
            quote! {
                #[automatically_derived]
                impl #impl_generics mem_dbg::CopyType for #name #ty_generics #where_clause
                {
                    type Copy = #copy_type;
                }

                #[automatically_derived]
                impl #impl_generics mem_dbg::MemSize for #name #ty_generics #where_clause {
                    fn mem_size(&self, _memsize_flags: mem_dbg::SizeFlags) -> usize {
                        let mut bytes = core::mem::size_of::<Self>();
                        #(bytes += self.#fields_ident.mem_size(_memsize_flags) - core::mem::size_of::<#fields_ty>();)*
                        bytes
                    }
                }
            }
        }

        Data::Enum(e) => {
            let mut variants = Vec::new();
            let mut variant_sizes = Vec::new();
            e.variants.iter().for_each(|variant| {
                let mut res = variant.ident.to_owned().to_token_stream();
                let mut var_args_size = quote! {core::mem::size_of::<Self>()};
                match &variant.fields {
                    syn::Fields::Unit => {}
                    syn::Fields::Named(fields) => {
                        let mut args = proc_macro2::TokenStream::new();
                        fields
                            .named
                            .iter()
                            .map(|field| {
                                let ty = &field.ty;
                                where_clause
                                    .predicates
                                    .push(parse_quote_spanned!(field.span()=> #ty: mem_dbg::MemSize));
                                (field.ident.as_ref().unwrap(), field.ty.to_token_stream())
                            })
                            .for_each(|(ident, ty)| {
                                var_args_size.extend([quote! {
                                    + #ident.mem_size(_memsize_flags) - core::mem::size_of::<#ty>()
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
                        fields.unnamed.iter().enumerate().for_each(|(field_idx, field)| {
                            let ident = syn::Ident::new(
                                &format!("v{}", field_idx),
                                proc_macro2::Span::call_site(),
                            )
                            .to_token_stream();
                            let ty = field.ty.to_token_stream();
                            var_args_size.extend([quote! {
                                + #ident.mem_size(_memsize_flags) - core::mem::size_of::<#ty>()
                            }]);
                            args.extend([ident]);
                            args.extend([quote! {,}]);

                            where_clause
                                .predicates
                                .push(parse_quote_spanned!(field.span()=> #ty: mem_dbg::MemSize));
                        });
                        // extend res with the args sourrounded by curly braces
                        res.extend(quote! {
                            ( #args )
                        });
                    }
                }
                variants.push(res);
                variant_sizes.push(var_args_size);
            });

            quote! {
                #[automatically_derived]
                impl #impl_generics mem_dbg::CopyType for #name #ty_generics #where_clause
                {
                    type Copy = #copy_type;
                }

                #[automatically_derived]
                impl #impl_generics mem_dbg::MemSize for #name #ty_generics #where_clause {
                    fn mem_size(&self, _memsize_flags: mem_dbg::SizeFlags) -> usize {
                        match self {
                            #(
                               #name::#variants => #variant_sizes,
                            )*
                        }
                    }
                }
            }
        }

        Data::Union(_) => unimplemented!("Unions are not supported"),
    }.into()
}

/**

Generate a `mem_dbg::MemDbg` implementation for custom types.

Presently we do not support unions.

*/
#[proc_macro_derive(MemDbg)]
pub fn mem_dbg_mem_dbg(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    input.generics.make_where_clause();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.unwrap().clone(); // We just created it

    match input.data {
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

                    let ty = &field.ty;
                    where_clause
                        .predicates
                        .push(parse_quote_spanned!(field.span()=> #ty: mem_dbg::MemDbgImpl));

                    let is_last = field_idx == s.fields.len().saturating_sub(1);

                    quote!{self.#field_ident.mem_dbg_depth_on(_memdbg_writer, _memdbg_total_size, _memdbg_depth, _memdbg_max_depth, Some(#fields_str), #is_last, _memdbg_flags)?;}
                })
                .collect::<Vec<_>>();

            quote! {
                #[automatically_derived]
                impl #impl_generics mem_dbg::MemDbgImpl for #name #ty_generics #where_clause {
                    #[inline(always)]
                    fn _mem_dbg_rec_on(
                        &self,
                        _memdbg_writer: &mut impl core::fmt::Write,
                        _memdbg_total_size: usize,
                        _memdbg_depth: usize,
                        _memdbg_max_depth: usize,
                        _memdbg_is_last: bool,
                        _memdbg_flags: mem_dbg::DbgFlags,
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
                            .for_each(|(idx, field)| {
                                let ident = field.ident.as_ref().unwrap();
                                let field_name = format!("{}", ident);
                                let is_last = idx == fields.named.len().saturating_sub(1);
                                variant_code.extend([quote! {
                                    #ident.mem_dbg_depth_on(_memdbg_writer, _memdbg_total_size, _memdbg_depth, _memdbg_max_depth, Some(#field_name), #is_last, _memdbg_flags)?;
                                }]);
                                args.extend([ident.to_token_stream()]);
                                args.extend([quote! {,}]);

                                let ty = &field.ty;
                                where_clause
                                    .predicates
                                    .push(parse_quote_spanned!(field.span()=> #ty: mem_dbg::MemDbgImpl));

                            });
                        // extend res with the args sourrounded by curly braces
                        res.extend(quote! {
                            { #args }
                        });
                    }
                    syn::Fields::Unnamed(fields) => {
                        let mut args = proc_macro2::TokenStream::new();
                        fields.unnamed.iter().enumerate().for_each(|(idx, field)| {
                            let ident = syn::Ident::new(
                                &format!("v{}", idx),
                                proc_macro2::Span::call_site(),
                            )
                            .to_token_stream();
                            let field_name = format!("{}", idx);
                            let is_last = idx == fields.unnamed.len().saturating_sub(1);
                            variant_code.extend([quote! {
                                #ident.mem_dbg_depth_on(_memdbg_writer, _memdbg_total_size, _memdbg_depth, _memdbg_max_depth, Some(#field_name), #is_last, _memdbg_flags)?;
                            }]);

                            args.extend([ident]);
                            args.extend([quote! {,}]);

                            let ty = &field.ty;
                            where_clause
                                .predicates
                                .push(parse_quote_spanned!(field.span()=> #ty: mem_dbg::MemDbgImpl));
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
                impl #impl_generics mem_dbg::MemDbgImpl  for #name #ty_generics #where_clause {
                    #[inline(always)]
                    fn _mem_dbg_rec_on(
                        &self,
                        _memdbg_writer: &mut impl core::fmt::Write,
                        _memdbg_total_size: usize,
                        _memdbg_depth: usize,
                        _memdbg_max_depth: usize,
                        _memdbg_is_last: bool,
                        _memdbg_flags: mem_dbg::DbgFlags,
                    ) -> core::fmt::Result {
                        let mut _memdbg_digits_number = mem_dbg::utils::n_of_digits(_memdbg_total_size);
                        if _memdbg_flags.contains(DbgFlags::SEPARATOR) {
                            _memdbg_digits_number += _memdbg_digits_number / 3;
                        }
                        if _memdbg_flags.contains(DbgFlags::HUMANIZE) {
                            _memdbg_digits_number = 6;
                        }
                        if _memdbg_flags.contains(DbgFlags::PERCENTAGE) {
                            _memdbg_digits_number = 5;
                        }

                        for _ in 0.._memdbg_digits_number + 3 {
                            _memdbg_writer.write_char(' ')?;
                        }
                        for _ in 0.._memdbg_depth.saturating_sub(1) {
                            _memdbg_writer.write_char('│')?;
                        }
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

        Data::Union(_) => unimplemented!("Unions are not supported"),
    }.into()
}
