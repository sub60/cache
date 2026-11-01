//! The derive macros of `sub60-cache-api`.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree};
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Data,
    DeriveInput,
    Field,
    Fields,
    Generics,
    Ident,
    LitStr,
    Token,
    Type,
    parse_macro_input,
    parse_quote,
};

/// Derives `PathParams` from a `#[route(..)]` attribute.
///
/// The attribute spells out the path as a sum of segments, each either a
/// type implementing `PathParams` or a string literal, e.g.
/// `#[route(CachePath + "/nar/{nar_file_name}")]`. The struct's fields must
/// follow the segments in order:
///
/// - a type takes the next field, which must be of that type;
/// - a string literal takes one field per `{name}` in it, which must be called
///   `name`, and whose type must implement `Display` and `FromStr`.
///
/// With the `axum` feature, it also implements axum's `FromRequestParts`, so
/// the struct can be extracted from requests directly. The expanded code
/// reaches axum through `sub60-cache-api`, so that crate's `axum` feature
/// must be enabled too, which is what enabling it does.
#[proc_macro_derive(PathParams, attributes(route))]
pub fn derive_path_params(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(&input).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// A segment of a `#[route(..)]` attribute.
#[expect(clippy::large_enum_variant)]
enum Segment {
    Type(Type),
    Literal(LitStr),
}

/// A piece of a string literal segment.
enum Piece {
    Text(String),
    Param(String),
}

impl Parse for Segment {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(LitStr) {
            input.parse().map(Self::Literal)
        } else {
            Type::without_plus(input).map(Self::Type)
        }
    }
}

#[expect(clippy::too_many_lines)]
fn expand(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let krate = quote!(::sub60_cache_api);

    let fields = named_fields(input)?;

    let attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("route"))
        .ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                "`PathParams` needs a `#[route(..)]` attribute",
            )
        })?;

    let segments = attr.parse_args_with(
        Punctuated::<Segment, Token![+]>::parse_separated_nonempty,
    )?;

    let generic_params = input
        .generics
        .type_params()
        .map(|param| param.ident.clone())
        .collect::<Vec<_>>();

    let mut fields = fields.iter();
    let mut path_parts = Vec::new();
    let mut write_stmts = Vec::new();
    let mut parse_stmts = Vec::new();
    let mut field_idents = Vec::new();

    for segment in &segments {
        match segment {
            Segment::Type(ty) => {
                if mentions_any(ty, &generic_params) {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "path segments can't depend on the struct's generic \
                         parameters",
                    ));
                }

                let field = fields.next().ok_or_else(|| {
                    syn::Error::new_spanned(
                        ty,
                        "missing a field for this path segment",
                    )
                })?;

                let ident = field_ident(field);
                let span = field.ty.span();

                path_parts.push(quote!(<#ty as #krate::PathParams>::PATH));
                write_stmts.push(quote_spanned! {span=>
                    <#ty as #krate::PathParams>::write_path(&self.#ident, path);
                });
                parse_stmts.push(quote_spanned! {span=>
                    let #ident: #ty =
                        <#ty as #krate::PathParams>::parse(params)?;
                });
                field_idents.push(ident.clone());
            },

            Segment::Literal(lit) => {
                for piece in pieces(lit)? {
                    match piece {
                        Piece::Text(text) => {
                            path_parts.push(quote!(#text));
                            write_stmts.push(quote!(path.push_str(#text);));
                        },
                        Piece::Param(name) => {
                            let field = fields.next().ok_or_else(|| {
                                syn::Error::new_spanned(
                                    lit,
                                    format!(
                                        "missing a field called `{name}` for \
                                         this path parameter"
                                    ),
                                )
                            })?;

                            let ident = field_ident(field);

                            if ident != name.as_str() {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    format!(
                                        "this field must be called `{name}`, \
                                         like the next parameter in the path"
                                    ),
                                ));
                            }

                            let template = format!("{{{name}}}");
                            path_parts.push(quote!(#template));
                            write_stmts.push(quote! {
                                #krate::__private::push_param(
                                    path,
                                    &self.#ident,
                                );
                            });
                            parse_stmts.push(quote! {
                                let #ident = #krate::__private::parse_param(
                                    params,
                                    #name,
                                )?;
                            });
                            field_idents.push(ident.clone());
                        },
                    }
                }
            },
        }
    }

    if let Some(field) = fields.next() {
        return Err(syn::Error::new_spanned(
            field_ident(field),
            "this field isn't in the path",
        ));
    }

    let name = &input.ident;
    let generics = with_param_bounds(&input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let extract_impl = if cfg!(feature = "axum") {
        let mut extract_generics = generics.clone();
        extract_generics.params.push(parse_quote!(
            __S: ::core::marker::Send + ::core::marker::Sync
        ));
        let (extract_impl_generics, _, _) = extract_generics.split_for_impl();

        quote! {
            impl #extract_impl_generics
                #krate::__private::axum::extract::FromRequestParts<__S>
                for #name #ty_generics
                #where_clause
            {
                type Rejection = #krate::PathParamsRejection;

                async fn from_request_parts(
                    parts: &mut #krate::__private::axum::http::request::Parts,
                    state: &__S,
                ) -> ::core::result::Result<Self, Self::Rejection> {
                    #krate::__private::from_request_parts(parts, state).await
                }
            }
        }
    } else {
        TokenStream2::new()
    };

    Ok(quote! {
        impl #impl_generics #krate::PathParams for #name #ty_generics
            #where_clause
        {
            const PATH: &'static str =
                #krate::__private::concatcp!(#(#path_parts),*);

            fn write_path(&self, path: &mut ::std::string::String) {
                #(#write_stmts)*
            }

            fn parse<'a>(
                params: &mut impl ::core::iter::Iterator<
                    Item = (&'a str, &'a str),
                >,
            ) -> ::core::result::Result<Self, #krate::PathParamsError> {
                #(#parse_stmts)*
                ::core::result::Result::Ok(Self { #(#field_idents),* })
            }
        }

        #extract_impl
    })
}

/// Returns the struct's fields, which must be named, or none for a unit
/// struct.
fn named_fields(
    input: &DeriveInput,
) -> syn::Result<Punctuated<Field, Token![,]>> {
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "`PathParams` can only be derived for structs",
        ));
    };

    match &data.fields {
        Fields::Named(fields) => Ok(fields.named.clone()),
        Fields::Unit => Ok(Punctuated::new()),
        Fields::Unnamed(fields) => Err(syn::Error::new_spanned(
            fields,
            "`PathParams` can't be derived for tuple structs",
        )),
    }
}

fn field_ident(field: &Field) -> &Ident {
    field.ident.as_ref().expect("fields are named")
}

/// Splits a string literal segment into text and `{name}` parameters.
fn pieces(lit: &LitStr) -> syn::Result<Vec<Piece>> {
    let value = lit.value();
    let mut pieces = Vec::new();
    let mut rest = value.as_str();

    while !rest.is_empty() {
        let Some(start) = rest.find(['{', '}']) else {
            pieces.push(Piece::Text(rest.to_owned()));
            break;
        };

        if rest[start..].starts_with('}') {
            return Err(syn::Error::new_spanned(lit, "unopened `}`"));
        }

        if start > 0 {
            pieces.push(Piece::Text(rest[..start].to_owned()));
        }

        let len = rest[start..]
            .find('}')
            .ok_or_else(|| syn::Error::new_spanned(lit, "unclosed `{`"))?;

        let name = &rest[start + 1..start + len];

        if syn::parse_str::<Ident>(name).is_err() {
            return Err(syn::Error::new_spanned(
                lit,
                format!("`{name}` isn't a valid parameter name"),
            ));
        }

        pieces.push(Piece::Param(name.to_owned()));
        rest = &rest[start + len + 1..];
    }

    Ok(pieces)
}

/// Returns whether the type mentions any of the given identifiers.
fn mentions_any(ty: &Type, idents: &[Ident]) -> bool {
    fn walk(tokens: TokenStream2, idents: &[Ident]) -> bool {
        tokens.into_iter().any(|token| match token {
            TokenTree::Ident(ident) => idents.contains(&ident),
            TokenTree::Group(group) => walk(group.stream(), idents),
            _ => false,
        })
    }

    walk(quote!(#ty), idents)
}

/// Adds the bounds path parameters need to each of the struct's type
/// parameters.
fn with_param_bounds(generics: &Generics) -> Generics {
    let mut generics = generics.clone();

    let params = generics
        .type_params()
        .map(|param| param.ident.clone())
        .collect::<Vec<_>>();

    let where_clause = generics.make_where_clause();

    for param in params {
        where_clause.predicates.push(parse_quote! {
            #param: ::core::fmt::Display + ::core::str::FromStr
        });
        where_clause.predicates.push(parse_quote! {
            <#param as ::core::str::FromStr>::Err: ::core::fmt::Display
        });
    }

    generics
}
