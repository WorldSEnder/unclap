//! Procedural macros for defining `Argument` and `Program`
extern crate proc_macro;
use attrs::{EnumVariant, FieldVariant, UnclapField, UnclapVariant};
use convert_case::{Case, Casing};
use darling::{util::Override, FromField, FromVariant};
use proc_macro::TokenStream as TS1;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort_call_site;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    ext::IdentExt, parse_macro_input, punctuated::Punctuated, spanned::Spanned, token::Comma,
    Attribute, Data, DataStruct, DeriveInput, Field, Fields, Ident, LitStr, Member, Variant,
};

mod attrs;

#[proc_macro_derive(Argument, attributes(argument))]
pub fn derive_argument(item: TS1) -> TS1 {
    let input: DeriveInput = parse_macro_input!(item);
    let res = do_derive_argument(&input);
    let _debug = format!("{:}", res);
    #[cfg(test)]
    eprintln!("{:}", _debug);
    res.into()
}

fn do_derive_argument(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;

    match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => unclap_for_struct(ident, &fields.named, &input.attrs),
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(ref fields),
            ..
        }) => unclap_for_struct(ident, &fields.unnamed, &input.attrs),
        Data::Enum(ref e) => unclap_for_enum(ident, &e.variants, &input.attrs),
        _ => abort_call_site!("`#[derive(Argument)]` only supports non-unit structs and enums"),
    }
}

fn flag_name_from_ident(ident: &Ident) -> LitStr {
    let flag_name = String::from("--") + &ident.unraw().to_string().to_case(Case::Kebab);
    LitStr::new(&flag_name, Span::call_site())
}

fn flag_name_for_field(field: &UnclapField, user: &Override<LitStr>) -> Option<LitStr> {
    user.clone()
        .explicit()
        .or_else(|| field.ident.as_ref().map(|id| flag_name_from_ident(id)))
}

fn flag_name_for_variant(variant: &UnclapVariant, user: &Override<LitStr>) -> LitStr {
    user.clone()
        .explicit()
        .unwrap_or_else(|| flag_name_from_ident(&variant.ident))
}

fn make_assert_impl_name(_field: &UnclapField, field_name: &Member) -> Ident {
    let mut ident = format_ident!("__unclap_assert{:}", field_name);
    ident.set_span(Span::mixed_site());
    ident
}

fn wrapped_field(
    field: &UnclapField,
    field_name: &Member,
    self_name: &Ident,
    receiver_name: &Ident,
) -> TokenStream {
    let field_ty = &field.ty;
    match &*field.variant {
        FieldVariant::Unnamed(..) => {
            let assert_name = make_assert_impl_name(field, field_name);
            let assert_arg = quote_spanned! {field_ty.span()=>
                #[allow(dead_code)]
                struct #assert_name where #field_ty: ::unclap_core::Argument;
            };
            quote! {
                #assert_arg
                <#field_ty as ::unclap_core::Argument>::append_to(&#self_name.#field_name, #receiver_name);
            }
        }
        FieldVariant::Named(name) => match flag_name_for_field(field, name) {
            Some(flag_name) => {
                let assert_name = make_assert_impl_name(field, field_name);
                let assert_arg = quote_spanned! {field_ty.span()=>
                    #[allow(dead_code)]
                    struct #assert_name where #field_ty: ::unclap_core::Argument;
                };
                quote! {
                    #assert_arg
                    ::unclap_support::Named::new(&#flag_name, &#self_name.#field_name)
                        .append_to(#receiver_name);
                }
            }
            None => quote_spanned! { field.variant.span()=>
                compile_error!("Can not use #[argument(variant(named))] on an unnamed field without providing a name");
            },
        },
        FieldVariant::Flag(name) => match flag_name_for_field(field, name) {
            Some(flag_name) => {
                let assert_name = make_assert_impl_name(field, field_name);
                let assert_arg = quote_spanned! {field_ty.span()=>
                    #[allow(dead_code)]
                    struct #assert_name where #field_ty: ::unclap_support::IsArgumentFlag;
                };
                quote! {
                    #assert_arg
                    ::unclap_support::FlagArg::new(&#flag_name, &#self_name.#field_name)
                        .append_to(#receiver_name);
                }
            }
            None => quote_spanned! { field.variant.span()=>
                compile_error!("Can not use #[argument(variant(flag))] on an unnamed field without providing a name");
            },
        },
    }
}

fn append_fields<'a, I: 'a + IntoIterator<Item = &'a Field>>(
    fields: I,
    self_name: &'a Ident,
    receiver_name: &'a Ident,
) -> impl 'a + Iterator<Item = TokenStream> {
    fields.into_iter().enumerate().map(move |(idx, field)| {
        let parsed_field = UnclapField::from_field(&field);
        let member_name = match field.ident.as_ref() {
            Some(name) => Member::Named(name.clone()),
            None => Member::Unnamed(idx.into()),
        };

        match parsed_field {
            Ok(parsed_field) => {
                wrapped_field(&parsed_field, &member_name, self_name, receiver_name)
            }
            Err(e) => e.write_errors(),
        }
    })
}

fn variant_prelude(variant: &UnclapVariant, receiver_name: &Ident) -> TokenStream {
    match &*variant.variant {
        EnumVariant::Unnamed(..) => quote! {},
        EnumVariant::Named(name) => {
            let flag_name = flag_name_for_variant(variant, name);
            quote! {
                #flag_name.append_to(#receiver_name);
            }
        }
    }
}

enum TupleMatchStyle {
    StyleUnit,
    StyleSingleTuple(UnclapField),
    NotAllowed(TokenStream),
}

fn unclap_for_variant(
    enum_name: &Ident,
    variant: &UnclapVariant,
    receiver_name: &Ident,
) -> TokenStream {
    let name = &variant.ident;
    let self_name = Ident::new("the_v", Span::call_site());
    let (tuple_match, style) = match variant.fields.style {
        darling::ast::Style::Unit => (quote! {}, TupleMatchStyle::StyleUnit),
        darling::ast::Style::Tuple => {
            if variant.fields.len() != 1 {
                let user_error = quote_spanned! {variant.span()=>
                    compile_error!("Enum variants with tuple structs must have exactly one field")
                };
                (quote! { (..) }, TupleMatchStyle::NotAllowed(user_error))
            } else {
                let field = UnclapField::from_field(&variant.fields.fields[0]);
                match field {
                    Ok(parsed_field) => (
                        quote! { ( #self_name ) },
                        TupleMatchStyle::StyleSingleTuple(parsed_field),
                    ),
                    Err(bad_field) => (
                        quote! { {..} },
                        TupleMatchStyle::NotAllowed(bad_field.write_errors()),
                    ),
                }
            }
        }
        darling::ast::Style::Struct => {
            let user_error = quote_spanned! {variant.span()=>
                compile_error!("Enum variants with named structs are not supported")
            };
            (quote! { {..} }, TupleMatchStyle::NotAllowed(user_error))
        }
    };
    match style {
        TupleMatchStyle::NotAllowed(tuple_error) => quote! {
            #enum_name :: #name #tuple_match => #tuple_error,
        },
        TupleMatchStyle::StyleSingleTuple(field) => {
            let field_ty = &field.ty;
            let assert_name = make_assert_impl_name(&field, &Member::Unnamed(0.into()));
            let assert_arg = quote_spanned! {variant.span()=>
                #[allow(dead_code)]
                struct #assert_name where #field_ty: ::unclap_core::Argument;
            };
            let append_field = quote! {
                #assert_arg
                <#field_ty as ::unclap_core::Argument>::append_to(&#self_name, #receiver_name);
            };

            let prelude = variant_prelude(&variant, &receiver_name);
            quote! {
                #enum_name :: #name #tuple_match => {
                    #prelude
                    #append_field
                },
            }
        }
        TupleMatchStyle::StyleUnit => {
            let prelude = variant_prelude(&variant, &receiver_name);
            quote! {
                #enum_name :: #name #tuple_match => {
                    #prelude
                },
            }
        }
    }
}

fn append_variant_arms<'a, I: 'a + IntoIterator<Item = &'a Variant>>(
    enum_name: &'a Ident,
    variants: I,
    receiver_name: &'a Ident,
) -> impl 'a + Iterator<Item = TokenStream> {
    variants.into_iter().map(move |v| {
        let parsed_var = UnclapVariant::from_variant(&v);

        match parsed_var {
            Ok(parsed_var) => unclap_for_variant(enum_name, &parsed_var, receiver_name),
            Err(e) => e.write_errors(),
        }
    })
}

fn unclap_for_struct(
    name: &Ident,
    fields: &Punctuated<Field, Comma>,
    _attrs: &[Attribute],
) -> TokenStream {
    let self_name = Ident::new("the_self", Span::call_site());
    let receiver_name = Ident::new("recv", Span::call_site());

    let fields = append_fields(fields, &self_name, &receiver_name);

    quote! {
        impl ::unclap_core::Argument for #name {
            fn append_to<R: ::unclap_core::ArgumentReceiver>(&self, #receiver_name: &mut R) {
                let #self_name = self;
                #( #fields )*
            }
        }
    }
}

fn unclap_for_enum(
    enum_name: &Ident,
    variants: &Punctuated<syn::Variant, Comma>,
    _attrs: &[Attribute],
) -> TokenStream {
    let receiver_name = Ident::new("recv", Span::call_site());
    let matches = append_variant_arms(enum_name, variants, &receiver_name);

    quote! {
        impl ::unclap_core::Argument for #enum_name {
            fn append_to<R: ::unclap_core::ArgumentReceiver>(&self, #receiver_name: &mut R) {
                match self {
                    #( #matches )*
                }
            }
        }
    }
}
