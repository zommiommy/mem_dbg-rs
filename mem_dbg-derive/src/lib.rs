/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Derive procedural macros for the [`mem_dbg`](https://crates.io/crates/mem_dbg) crate.

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Data, DeriveInput, parse_macro_input, parse_quote, parse_quote_spanned, spanned::Spanned,
};

/// Generate a `mem_dbg::MemSize` implementation for custom types.
///
/// Presently we do not support unions.
///
/// The attribute `#[mem_size(flat)]` can be used on flat types (typically `Copy +
/// 'static`) that do not contain non-`'static` references to make
/// `MemSize::mem_size` faster on arrays, vectors, slices, and supported
/// containers.
///
/// When all fields implement `FlatType<Flat=True>` but neither
/// `#[mem_size(flat)]` nor `#[mem_size(rec)]` is present, a compile-time error
/// is emitted. Use `#[mem_size(rec)]` to explicitly silence this check when the
/// type is intentionally not `#[mem_size(flat)]`.
///
/// See `mem_dbg::FlatType` for more details.
#[proc_macro_derive(MemSize, attributes(mem_size, mem_size_flat, mem_size_rec))]
pub fn mem_dbg_mem_size(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let input_ident = input.ident;
    input.generics.make_where_clause();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.unwrap().clone(); // We just created it

    let mut is_flat = false;
    let mut is_rec = false;

    for attr in &input.attrs {
        if attr.meta.path().is_ident("mem_size_flat") {
            is_flat = true;
            eprintln!(
                "warning: use `#[mem_size(flat)]` instead of `#[mem_size_flat]` on type `{input_ident}`"
            );
        } else if attr.meta.path().is_ident("mem_size_rec") {
            is_rec = true;
            eprintln!(
                "warning: use `#[mem_size(rec)]` instead of `#[mem_size_rec]` on type `{input_ident}`"
            );
        } else if attr.meta.path().is_ident("mem_size") {
            if let Err(e) = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("flat") {
                    is_flat = true;
                    Ok(())
                } else if meta.path.is_ident("rec") {
                    is_rec = true;
                    Ok(())
                } else {
                    Err(meta.error("expected `flat` or `rec`"))
                }
            }) {
                return e.to_compile_error().into();
            }
        }
    }

    if is_flat && is_rec {
        return syn::Error::new_spanned(
            &input_ident,
            "cannot use both `flat` and `rec` on the same type",
        )
        .to_compile_error()
        .into();
    }

    let flat_type: syn::Expr = if is_flat {
        parse_quote!(::mem_dbg::True)
    } else {
        parse_quote!(::mem_dbg::False)
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
                // Add MemSize and FlatType bounds to all fields
                where_clause
                    .predicates
                    .push(parse_quote_spanned!(field.span()=> #field_ty: ::mem_dbg::MemSize + ::mem_dbg::FlatType));
            }

            let const_assert = if !is_flat && !is_rec {
                let msg = format!(
                    "Structure {} could be #[mem_size(flat)], but it has not been declared as such; use either the #[mem_size(flat)] or the #[mem_size(rec)] attribute to silence this error",
                    input_ident
                );
                quote! {
                    const { assert!(
                        !(true #(&& <<#fields_ty as ::mem_dbg::FlatType>::Flat
                                      as ::mem_dbg::Boolean>::VALUE)*),
                        #msg
                    ); }
                }
            } else {
                quote! {}
            };

            quote! {
                #[automatically_derived]
                impl #impl_generics ::mem_dbg::FlatType for #input_ident #ty_generics #where_clause
                {
                    type Flat = #flat_type;
                }

                #[automatically_derived]
                impl #impl_generics ::mem_dbg::MemSize for #input_ident #ty_generics #where_clause {
                    fn mem_size_rec(&self, _memsize_flags: ::mem_dbg::SizeFlags, _memsize_refs: &mut ::mem_dbg::HashMap<usize, usize>) -> usize {
                        #const_assert
                        let mut bytes = ::core::mem::size_of::<Self>();
                        #(bytes += <#fields_ty as ::mem_dbg::MemSize>::mem_size_rec(&self.#fields_ident, _memsize_flags, _memsize_refs) - ::core::mem::size_of::<#fields_ty>();)*
                        bytes
                    }
                }
            }
        }

        Data::Enum(e) => {
            let mut variants = Vec::new();
            let mut variants_size = Vec::new();
            let mut all_field_types = Vec::new();

            for variant in e.variants {
                let mut res = variant.ident.to_owned().to_token_stream();
                let mut var_args_size = quote! {::core::mem::size_of::<Self>()};
                match &variant.fields {
                    syn::Fields::Unit => {}
                    syn::Fields::Named(fields) => {
                        let mut args = proc_macro2::TokenStream::new();
                        for field in &fields.named {
                            let field_ty = &field.ty;
                            where_clause
                                .predicates
                                .push(parse_quote_spanned!(field.span() => #field_ty: ::mem_dbg::MemSize + ::mem_dbg::FlatType));
                            if !is_flat && !is_rec {
                                all_field_types.push(field.ty.to_token_stream());
                            }
                            let field_ident = field.ident.as_ref().unwrap();
                            // Use a prefixed binding to avoid shadowing
                            // generated locals.
                            let binding_ident = syn::Ident::new(
                                &format!("_memsize_{}", field_ident),
                                field_ident.span(),
                            );
                            let field_ty = field.ty.to_token_stream();
                            var_args_size.extend([quote! {
                                + <#field_ty as ::mem_dbg::MemSize>::mem_size_rec(#binding_ident, _memsize_flags, _memsize_refs) - ::core::mem::size_of::<#field_ty>()
                            }]);
                            args.extend([quote! { #field_ident: #binding_ident, }]);
                        }
                        // Extend res with the args surrounded by curly braces
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
                                + <#field_ty as ::mem_dbg::MemSize>::mem_size_rec(#ident, _memsize_flags, _memsize_refs) - ::core::mem::size_of::<#field_ty>()
                            }]);
                            args.extend([ident]);
                            args.extend([quote! {,}]);

                            where_clause
                                .predicates
                                .push(parse_quote_spanned!(field.span()=> #field_ty: ::mem_dbg::MemSize + ::mem_dbg::FlatType));
                            if !is_flat && !is_rec {
                                all_field_types.push(field.ty.to_token_stream());
                            }
                        }
                        // extend res with the args surrounded by curly braces
                        res.extend(quote! {
                            ( #args )
                        });
                    }
                }
                variants.push(res);
                variants_size.push(var_args_size);
            }

            let const_assert = if !is_flat && !is_rec {
                let msg = format!(
                    "Enum {} could be #[mem_size(flat)], but it has not been declared as such; use either the #[mem_size(flat)] or the #[mem_size(rec)] attribute to silence this error",
                    input_ident
                );
                quote! {
                    const { assert!(
                        !(true #(&& <<#all_field_types as ::mem_dbg::FlatType>::Flat
                                      as ::mem_dbg::Boolean>::VALUE)*),
                        #msg
                    ); }
                }
            } else {
                quote! {}
            };

            quote! {
                #[automatically_derived]
                impl #impl_generics ::mem_dbg::FlatType for #input_ident #ty_generics #where_clause
                {
                    type Flat = #flat_type;
                }

                #[automatically_derived]
                impl #impl_generics ::mem_dbg::MemSize for #input_ident #ty_generics #where_clause {
                    fn mem_size_rec(&self, _memsize_flags: ::mem_dbg::SizeFlags, _memsize_refs: &mut ::mem_dbg::HashMap<usize, usize>) -> usize {
                        #const_assert
                        match self {
                            #(
                               #input_ident::#variants => #variants_size,
                            )*
                        }
                    }
                }
            }
        }

        Data::Union(u) => {
            return syn::Error::new_spanned(u.union_token, "MemSize for unions is not supported; see the Unions section in the README for a manual implementation pattern")
                .to_compile_error()
                .into();
        }
    }.into()
}

/// Generate a `mem_dbg::MemDbg` implementation for custom types.
///
/// Presently we do not support unions.
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
                    .push(parse_quote_spanned!(field.span() => #field_ty: ::mem_dbg::MemDbgImpl + ::mem_dbg::FlatType));

                // We push the field index, its offset, and its size
                // (the size is used to break ties when sorting by offset,
                // ensuring ZSTs come before non-ZSTs at the same offset)
                id_offset_pushes.push(quote!{
                    id_sizes.push((#field_idx, ::core::mem::offset_of!(#input_ident #ty_generics, #field_ident), ::core::mem::size_of::<#field_ty>()));
                });
                // This is the arm of the match statement that invokes
                // _mem_dbg_depth_on on the field.
                match_code.push(quote!{
                    #field_idx => <#field_ty as ::mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(&self.#field_ident, _memdbg_writer, _memdbg_total_size, _memdbg_max_depth, _memdbg_prefix, Some(#field_ident_str), i == n - 1, padded_size, _memdbg_flags, _memdbg_refs)?,
                });
            }

            quote! {
                #[automatically_derived]
                impl #impl_generics ::mem_dbg::MemDbgImpl for #input_ident #ty_generics #where_clause {
                    fn _mem_dbg_rec_on(
                        &self,
                        _memdbg_writer: &mut impl ::core::fmt::Write,
                        _memdbg_total_size: usize,
                        _memdbg_max_depth: usize,
                        _memdbg_prefix: &mut String,
                        _memdbg_is_last: bool,
                        _memdbg_flags: ::mem_dbg::DbgFlags,
                        _memdbg_refs: &mut ::mem_dbg::HashSet<usize>,
                    ) -> ::core::fmt::Result {
                        let mut id_sizes: Vec<(usize, usize, usize)> = vec![];
                        #(#id_offset_pushes)*
                        let n = id_sizes.len();
                        id_sizes.push((n, ::core::mem::size_of::<Self>(), usize::MAX));
                        // Sort by offset, breaking ties by size so ZSTs come
                        // before non-ZSTs at the same offset
                        id_sizes.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.cmp(&b.2)));
                        // Compute padded sizes
                        for i in 0..n {
                            id_sizes[i].1 = id_sizes[i + 1].1 - id_sizes[i].1;
                        };
                        // Put the candle back unless the user requested otherwise
                        if ! _memdbg_flags.contains(::mem_dbg::DbgFlags::RUST_LAYOUT) {
                            id_sizes.sort_by_key(|x| x.0);
                        }

                        for (i, (field_idx, padded_size, _)) in id_sizes.into_iter().enumerate().take(n) {
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
                            let field_ty = &field.ty;
                            let field_ident = field.ident.as_ref().unwrap();
                            let field_ident_str = format!("{}", field_ident);
                            // Use a prefixed binding to avoid shadowing
                            // generated locals such as `n`, `i`, etc.
                            let binding_ident = syn::Ident::new(
                                &format!("_memdbg_{}", field_ident),
                                field_ident.span(),
                            );

                            #[cfg(feature = "offset_of_enum")]
                            id_offset_pushes.push({
                                let variant_ident = &variant.ident;
                                quote!{
                                    // We push the offset and size of the field;
                                    // the size is used to break ties when sorting
                                    // by offset (ZSTs before non-ZSTs).
                                    id_sizes.push((#field_idx, ::core::mem::offset_of!(#input_ident #ty_generics, #variant_ident . #field_ident), ::core::mem::size_of_val(#binding_ident)));
                                }
                            });
                            #[cfg(not(feature = "offset_of_enum"))]
                            id_offset_pushes.push(quote!{
                                // We push the size of the field, which will be
                                // used as a surrogate of the padded size.
                                id_sizes.push((#field_idx, ::core::mem::size_of_val(#binding_ident)));
                            });

                            // This is the arm of the match statement that
                            // invokes _mem_dbg_depth_on on the field.
                            match_code.push(quote! {
                                #field_idx => <#field_ty as ::mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(#binding_ident, _memdbg_writer, _memdbg_total_size, _memdbg_max_depth, _memdbg_prefix, Some(#field_ident_str), i == n - 1, padded_size, _memdbg_flags, _memdbg_refs)?,
                            });
                            args.extend([quote! { #field_ident: #binding_ident, }]);

                            let field_ty = &field.ty;
                            where_clause
                                .predicates
                                .push(parse_quote_spanned!(field.span()=> #field_ty: ::mem_dbg::MemDbgImpl + ::mem_dbg::FlatType));
                        }
                        // Extend res with the args surrounded by curly braces
                        res.extend(quote! {
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
                            let field_ty = &field.ty;
                            let field_ident_str = format!("{}", field_idx);
                            let _field_tuple_idx = syn::Index::from(field_idx);

                            #[cfg(feature = "offset_of_enum")]
                            id_offset_pushes.push({
                                let variant_ident = &variant.ident;
                                quote!{
                                    // We push the offset and size of the field;
                                    // the size is used to break ties when sorting
                                    // by offset (ZSTs before non-ZSTs).
                                    id_sizes.push((#field_idx, ::core::mem::offset_of!(#input_ident #ty_generics, #variant_ident . #_field_tuple_idx), ::core::mem::size_of_val(#field_ident)));
                                }
                            });

                            #[cfg(not(feature = "offset_of_enum"))]
                            id_offset_pushes.push(quote!{
                                // We push the size of the field, which will be
                                // used as a surrogate of the padded size.
                                id_sizes.push((#field_idx, ::core::mem::size_of_val(#field_ident)));
                            });

                            // This is the arm of the match statement that
                            // invokes _mem_dbg_depth_on on the field.
                            match_code.push(quote! {
                                #field_idx => <#field_ty as ::mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(#field_ident, _memdbg_writer, _memdbg_total_size, _memdbg_max_depth, _memdbg_prefix, Some(#field_ident_str), i == n - 1, padded_size, _memdbg_flags, _memdbg_refs)?,
                            });

                            args.extend([field_ident]);
                            args.extend([quote! {,}]);

                            let field_ty = &field.ty;
                            where_clause
                                .predicates
                                .push(parse_quote_spanned!(field.span()=> #field_ty: ::mem_dbg::MemDbgImpl + ::mem_dbg::FlatType));
                        }
                        // extend res with the args surrounded by curly braces
                        res.extend(quote! {
                            ( #args )
                        });
                    }
                }
                variants.push(res);
                let variant_name = format!("Variant: {}\n", variant.ident);

                // There's some code duplication here, but we need to keep the
                // #[cfg] attributes outside of the quote! macro.
                // IMPORTANT: We must push exactly ONE item to variants_code per
                // variant to match the length of the variants Vec.

                #[cfg(feature = "offset_of_enum")]
                variants_code.push(quote!{{
                    _memdbg_writer.write_char(#arrow)?;
                    _memdbg_writer.write_char('╴')?;
                    _memdbg_writer.write_str(#variant_name)?;

                    let mut id_sizes: Vec<(usize, usize, usize)> = vec![];
                    #(#id_offset_pushes)*
                    let n = id_sizes.len();

                    // We use the offset_of information to build the real
                    // space occupied by a field.
                    id_sizes.push((n, ::core::mem::size_of::<Self>(), usize::MAX));
                    // Sort by offset, breaking ties by size so ZSTs come
                    // before non-ZSTs at the same offset
                    id_sizes.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.cmp(&b.2)));
                    // Compute padded sizes
                    for i in 0..n {
                        id_sizes[i].1 = id_sizes[i + 1].1 - id_sizes[i].1;
                    };
                    // Put the candle back unless the user requested otherwise
                    if ! _memdbg_flags.contains(::mem_dbg::DbgFlags::RUST_LAYOUT) {
                        id_sizes.sort_by_key(|x| x.0);
                    }

                    for (i, (field_idx, padded_size, _)) in id_sizes.into_iter().enumerate().take(n) {
                        match field_idx {
                            #(#match_code)*
                            _ => unreachable!(),
                        }
                    }
                }});

                #[cfg(not(feature = "offset_of_enum"))]
                variants_code.push(quote!{{
                    _memdbg_writer.write_char(#arrow)?;
                    _memdbg_writer.write_char('╴')?;
                    _memdbg_writer.write_str(#variant_name)?;

                    let mut id_sizes: Vec<(usize, usize)> = vec![];
                    #(#id_offset_pushes)*
                    let n = id_sizes.len();

                    // Lacking offset_of for enums, id_sizes contains the
                    // size_of of each field which we use as a surrogate of
                    // the padded size.
                    assert!(!_memdbg_flags.contains(::mem_dbg::DbgFlags::RUST_LAYOUT), "DbgFlags::RUST_LAYOUT for enums requires the offset_of_enum feature");

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
                impl #impl_generics ::mem_dbg::MemDbgImpl  for #input_ident #ty_generics #where_clause {
                    fn _mem_dbg_rec_on(
                        &self,
                        _memdbg_writer: &mut impl ::core::fmt::Write,
                        _memdbg_total_size: usize,
                        _memdbg_max_depth: usize,
                        _memdbg_prefix: &mut String,
                        _memdbg_is_last: bool,
                        _memdbg_flags: ::mem_dbg::DbgFlags,
                        _memdbg_refs: &mut ::mem_dbg::HashSet<usize>,
                    ) -> ::core::fmt::Result {
                        let mut _memdbg_digits_number = ::mem_dbg::n_of_digits(_memdbg_total_size);
                        if _memdbg_flags.contains(::mem_dbg::DbgFlags::SEPARATOR) {
                            _memdbg_digits_number += _memdbg_digits_number / 3;
                        }
                        if _memdbg_flags.contains(::mem_dbg::DbgFlags::HUMANIZE) {
                            _memdbg_digits_number = 6;
                        }

                        if _memdbg_flags.contains(::mem_dbg::DbgFlags::PERCENTAGE) {
                            _memdbg_digits_number += 8;
                        }

                        for _ in 0.._memdbg_digits_number + 3 {
                            _memdbg_writer.write_char(' ')?;
                        }
                        if !_memdbg_prefix.is_empty() {
                            // Find the byte index of the 3rd character (skip first 2 chars)
                            // to handle multi-byte UTF-8 characters like "│"
                            let start_byte = _memdbg_prefix
                                .char_indices()
                                .nth(2)
                                .map(|(idx, _)| idx)
                                .unwrap_or(_memdbg_prefix.len());
                            _memdbg_writer.write_str(&_memdbg_prefix[start_byte..])?;
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

        Data::Union(u) => {
            return syn::Error::new_spanned(u.union_token, "MemDbg for unions is not supported; see the Unions section in the README for a manual implementation pattern")
                .to_compile_error()
                .into();
        }
    }.into()
}
