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

The attribute `copy_type` can be used on [`Copy`] types that do not contain non-`'static` references
to make `MemSize::mem_size` faster on arrays, vectors and slices. Note that specifying
`copy_type` will add the bound that the type is `Copy + 'static`.

See `mem_dbg::CopyType` for more details.

*/
#[proc_macro_derive(MemSize, attributes(copy_type))]
pub fn mem_dbg_mem_size(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let input_ident = input.ident;
    input.generics.make_where_clause();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.unwrap().clone(); // We just created it

    let is_copy_type = input
        .attrs
        .iter()
        .any(|x| x.meta.path().is_ident("copy_type"));

    // If copy_type, add the Copy + 'static bound
    let copy_type: syn::Expr = if is_copy_type {
        where_clause
            .predicates
            .push(parse_quote_spanned!(input_ident.span()=> Self: Copy + 'static));
        parse_quote!(mem_dbg::True)
    } else {
        parse_quote!(mem_dbg::False)
    };

    match input.data {
        Data::Struct(s) => {
            let mut fields_ident = vec![];
            let mut fields_ty = vec![];

            for (field_idx, field) in s.fields.iter().enumerate() {
                fields_ident.push(
                    field
                        .ident
                        .to_owned()
                        .map(|t| t.to_token_stream())
                        .unwrap_or(syn::Index::from(field_idx).to_token_stream()),
                );
                fields_ty.push(field.ty.to_token_stream());
                let field_ty = &field.ty;
                // Add MemSize bound to all fields
                where_clause
                    .predicates
                    .push(parse_quote_spanned!(field.span()=> #field_ty: mem_dbg::MemSize));
            }
            quote! {
                #[automatically_derived]
                impl #impl_generics mem_dbg::CopyType for #input_ident #ty_generics #where_clause
                {
                    type Copy = #copy_type;
                }

                #[automatically_derived]
                impl #impl_generics mem_dbg::MemSize for #input_ident #ty_generics #where_clause {
                    fn mem_size(&self, _memsize_flags: mem_dbg::SizeFlags) -> usize {
                        let mut bytes = core::mem::size_of::<Self>();
                        #(bytes += <#fields_ty as mem_dbg::MemSize>::mem_size(&self.#fields_ident, _memsize_flags) - core::mem::size_of::<#fields_ty>();)*
                        bytes
                    }
                }
            }
        }

        Data::Enum(e) => {
            let mut variants = Vec::new();
            let mut variants_size = Vec::new();

            for variant in e.variants {
                let mut res = variant.ident.to_owned().to_token_stream();
                let mut var_args_size = quote! {core::mem::size_of::<Self>()};
                match &variant.fields {
                    syn::Fields::Unit => {}
                    syn::Fields::Named(fields) => {
                        let mut args = proc_macro2::TokenStream::new();
                        for field in &fields.named {
                            let field_ty = &field.ty;
                            where_clause
                                .predicates
                                .push(parse_quote_spanned!(field.span() => #field_ty: mem_dbg::MemSize));
                                let field_ident = &field.ident;
                                let field_ty = field.ty.to_token_stream();
                                var_args_size.extend([quote! {
                                    + <#field_ty as mem_dbg::MemSize>::mem_size(#field_ident, _memsize_flags) - core::mem::size_of::<#field_ty>()
                                }]);
                                args.extend([field_ident.to_token_stream()]);
                                args.extend([quote! {,}]);
                            }
                        // extend res with the args sourrounded by curly braces
                        res.extend(quote! {
                            { #args }
                        });
                    }
                    syn::Fields::Unnamed(fields) => {
                        let mut args = proc_macro2::TokenStream::new();

                        for (field_idx, field) in fields.unnamed.iter().enumerate() {
                            let ident = syn::Ident::new(
                                &format!("v{}", field_idx),
                                proc_macro2::Span::call_site(),
                            )
                            .to_token_stream();
                            let field_ty = field.ty.to_token_stream();
                            var_args_size.extend([quote! {
                                + <#field_ty as mem_dbg::MemSize>::mem_size(#ident, _memsize_flags) - core::mem::size_of::<#field_ty>()
                            }]);
                            args.extend([ident]);
                            args.extend([quote! {,}]);

                            where_clause
                                .predicates
                                .push(parse_quote_spanned!(field.span()=> #field_ty: mem_dbg::MemSize));
                        }
                        // extend res with the args sourrounded by curly braces
                        res.extend(quote! {
                            ( #args )
                        });
                    }
                }
                variants.push(res);
                variants_size.push(var_args_size);
            }

            quote! {
                #[automatically_derived]
                impl #impl_generics mem_dbg::CopyType for #input_ident #ty_generics #where_clause
                {
                    type Copy = #copy_type;
                }

                #[automatically_derived]
                impl #impl_generics mem_dbg::MemSize for #input_ident #ty_generics #where_clause {
                    fn mem_size(&self, _memsize_flags: mem_dbg::SizeFlags) -> usize {
                        match self {
                            #(
                               #input_ident::#variants => #variants_size,
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

    let input_ident = input.ident;
    input.generics.make_where_clause();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.unwrap().clone(); // We just created it

    match input.data {
        Data::Struct(s) => {
            let mut id_offset_pushes = vec![];
            let mut match_code = vec![];

            for (field_idx, field) in s.fields.iter().enumerate() {
                // Use the field name for named structures, and the index
                // for tuple structures
                let field_ident = field
                    .ident
                    .to_owned()
                    .map(|t| t.to_token_stream())
                    .unwrap_or_else(|| syn::Index::from(field_idx).to_token_stream());

                let field_ident_str = field
                    .ident
                    .to_owned()
                    .map(|t| t.to_string().to_token_stream())
                    .unwrap_or_else(|| field_idx.to_string().to_token_stream());

                let field_ty = &field.ty;
                where_clause
                    .predicates
                    .push(parse_quote_spanned!(field.span() => #field_ty: mem_dbg::MemDbgImpl));

                // We push the field index and its offset
                id_offset_pushes.push(quote!{
                    id_sizes.push((#field_idx, core::mem::offset_of!(#input_ident #ty_generics, #field_ident)));
                });
                // This is the arm of the match statement that invokes
                // _mem_dbg_depth_on on the field.
                match_code.push(quote!{
                    #field_idx => self.#field_ident._mem_dbg_depth_on(_memdbg_writer, _memdbg_total_size, _memdbg_max_depth, _memdbg_prefix, Some(#field_ident_str), i == n - 1, padded_size, _memdbg_flags)?,
                });
            }

            quote! {
                #[automatically_derived]
                impl #impl_generics mem_dbg::MemDbgImpl for #input_ident #ty_generics #where_clause {
                    #[inline(always)]
                    fn _mem_dbg_rec_on(
                        &self,
                        _memdbg_writer: &mut impl core::fmt::Write,
                        _memdbg_total_size: usize,
                        _memdbg_max_depth: usize,
                        _memdbg_prefix: &mut String,
                        _memdbg_is_last: bool,
                        _memdbg_flags: mem_dbg::DbgFlags,
                    ) -> core::fmt::Result {
                        let mut id_sizes: Vec<(usize, usize)> = vec![];
                        #(#id_offset_pushes)*
                        let n = id_sizes.len();
                        id_sizes.push((n, core::mem::size_of::<Self>()));
                        // Sort by offset
                        id_sizes.sort_by_key(|x| x.1);
                        // Compute padded sizes
                        for i in 0..n {
                            id_sizes[i].1 = id_sizes[i + 1].1 - id_sizes[i].1;
                        };
                        // Put the candle back unless the user requested otherwise
                        if ! _memdbg_flags.contains(mem_dbg::DbgFlags::RUST_LAYOUT) {
                            id_sizes.sort_by_key(|x| x.0);
                        }

                        for (i, (field_idx, padded_size)) in id_sizes.into_iter().enumerate().take(n) {
                            match field_idx {
                                #(#match_code)*
                                _ => unreachable!(),
                            }
                        }
                        Ok(())
                    }
                }
            }
        }

        Data::Enum(e) => {
            let mut variants = Vec::new();
            let mut variants_code = Vec::new();

            for variant in &e.variants {
                let variant_ident = &variant.ident;
                let mut res = variant.ident.to_owned().to_token_stream();
                // Depending on the presence of the feature offset_of_enum, this
                // will contains field indices and offset_of or field indices
                // and size_of; in the latter case, we will assume size_of to be
                // the padded size, resulting in no padding.
                let mut id_offset_pushes = vec![];
                let mut match_code = vec![];
                let mut arrow = '╰';
                match &variant.fields {
                    syn::Fields::Unit => {},
                    syn::Fields::Named(fields) => {
                        let mut args = proc_macro2::TokenStream::new();
                        if !fields.named.is_empty() {
                            arrow = '├';
                        }
                        for (field_idx, field) in fields.named.iter().enumerate() {
                            let field_ident = field.ident.as_ref().unwrap();
                            let field_ident_str = format!("{}", field_ident);
                            id_offset_pushes.push(quote!{
                                // We push the offset of the field, which will
                                // be used to compute the padded size.
                                #[cfg(feature = "offset_of_enum")]
                                id_sizes.push((#field_idx, core::mem::offset_of!(#input_ident #ty_generics, #variant_ident . #field_ident)));
                                // We push the size of the field, which will be
                                // used as a surrogate of the padded size.
                                #[cfg(not(feature = "offset_of_enum"))]
                                id_sizes.push((#field_idx, std::mem::size_of_val(#field_ident)));
                            });

                            // This is the arm of the match statement that
                            // invokes _mem_dbg_depth_on on the field.
                            match_code.push(quote! {
                                #field_idx => #field_ident._mem_dbg_depth_on(_memdbg_writer, _memdbg_total_size, _memdbg_max_depth, _memdbg_prefix, Some(#field_ident_str), i == n - 1, padded_size, _memdbg_flags)?,
                            });
                            args.extend([field_ident.to_token_stream()]);
                            args.extend([quote! {,}]);

                            let field_ty = &field.ty;
                            where_clause
                                .predicates
                                .push(parse_quote_spanned!(field.span()=> #field_ty: mem_dbg::MemDbgImpl));
                        }
                        // extend res with the args sourrounded by curly braces
                        res.extend(quote! {
                            // TODO: sanitize somehow the names or it'll be
                            // havoc.
                            { #args }
                        });
                    }
                    syn::Fields::Unnamed(fields) => {
                        let mut args = proc_macro2::TokenStream::new();
                        if !fields.unnamed.is_empty() {
                            arrow = '├';
                        }
                        for (field_idx, field) in fields.unnamed.iter().enumerate() {
                            let field_ident = syn::Ident::new(
                                &format!("v{}", field_idx),
                                proc_macro2::Span::call_site(),
                            )
                            .to_token_stream();
                            let field_ident_str = format!("{}", field_idx);
                            let field_tuple_idx = syn::Index::from(field_idx);

                            id_offset_pushes.push(quote!{
                                // We push the offset of the field, which will
                                // be used to compute the padded size.
                                #[cfg(feature = "offset_of_enum")]
                                id_sizes.push((#field_idx, core::mem::offset_of!(#input_ident #ty_generics, #variant_ident . #field_tuple_idx)));
                                // We push the size of the field, which will be
                                // used as a surrogate of the padded size.
                                #[cfg(not(feature = "offset_of_enum"))]
                                id_sizes.push((#field_idx, std::mem::size_of_val(#field_ident)));
                            });

                            // This is the arm of the match statement that
                            // invokes _mem_dbg_depth_on on the field.
                            match_code.push(quote! {
                                #field_idx => #field_ident._mem_dbg_depth_on(_memdbg_writer, _memdbg_total_size, _memdbg_max_depth, _memdbg_prefix, Some(#field_ident_str), i == n - 1, padded_size, _memdbg_flags)?,
                            });

                            args.extend([field_ident]);
                            args.extend([quote! {,}]);

                            let field_ty = &field.ty;
                            where_clause
                                .predicates
                                .push(parse_quote_spanned!(field.span()=> #field_ty: mem_dbg::MemDbgImpl));
                        }
                        // extend res with the args sourrounded by curly braces
                        res.extend(quote! {
                            ( #args )
                        });
                    }
                }
                variants.push(res);
                let variant_name = format!("Variant: {}\n", variant.ident);
                variants_code.push(quote!{{
                    _memdbg_writer.write_char(#arrow)?;
                    _memdbg_writer.write_char('╴')?;
                    _memdbg_writer.write_str(#variant_name)?;

                    let mut id_sizes: Vec<(usize, usize)> = vec![];
                    #(#id_offset_pushes)*
                    let n = id_sizes.len();
                    #[cfg(feature = "offset_of_enum")]
                    {
                        // We use the offset_of information to build the real
                        // space occupied by a field.
                        id_sizes.push((n, core::mem::size_of::<Self>()));
                        // Sort by offset
                        id_sizes.sort_by_key(|x| x.1);
                        // Compute padded sizes
                        for i in 0..n {
                            id_sizes[i].1 = id_sizes[i + 1].1 - id_sizes[i].1;
                        };
                        // Put the candle back unless the user requested otherwise
                        if ! _memdbg_flags.contains(mem_dbg::DbgFlags::RUST_LAYOUT) {
                            id_sizes.sort_by_key(|x| x.0);
                        }
                    }
                    #[cfg(not(feature = "offset_of_enum"))]
                    {
                        // Lacking offset_of for enums, id_sizes contains the
                        // size_of of each field which we use as a surrogate of
                        // the padded size.
                        assert!(!_memdbg_flags.contains(mem_dbg::DbgFlags::RUST_LAYOUT), "DbgFlags::RUST_LAYOUT for enums requires the offset_of_enum feature");
                    }
                    for (i, (field_idx, padded_size)) in id_sizes.into_iter().enumerate().take(n) {
                        match field_idx {
                            #(#match_code)*
                            _ => unreachable!(),
                        }
                    }

                }});
            }

            quote! {
                #[automatically_derived]
                impl #impl_generics mem_dbg::MemDbgImpl  for #input_ident #ty_generics #where_clause {
                    #[inline(always)]
                    fn _mem_dbg_rec_on(
                        &self,
                        _memdbg_writer: &mut impl core::fmt::Write,
                        _memdbg_total_size: usize,
                        _memdbg_max_depth: usize,
                        _memdbg_prefix: &mut String,
                        _memdbg_is_last: bool,
                        _memdbg_flags: mem_dbg::DbgFlags,
                    ) -> core::fmt::Result {
                        let mut _memdbg_digits_number = mem_dbg::n_of_digits(_memdbg_total_size);
                        if _memdbg_flags.contains(mem_dbg::DbgFlags::SEPARATOR) {
                            _memdbg_digits_number += _memdbg_digits_number / 3;
                        }
                        if _memdbg_flags.contains(mem_dbg::DbgFlags::HUMANIZE) {
                            _memdbg_digits_number = 6;
                        }

                        if _memdbg_flags.contains(mem_dbg::DbgFlags::PERCENTAGE) {
                            _memdbg_digits_number += 8;
                        }

                        for _ in 0.._memdbg_digits_number + 3 {
                            _memdbg_writer.write_char(' ')?;
                        }
                        if !_memdbg_prefix.is_empty() {
                            _memdbg_writer.write_str(&_memdbg_prefix[2..])?;
                        }
                        match self {
                            #(
                               #input_ident::#variants => #variants_code,
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
