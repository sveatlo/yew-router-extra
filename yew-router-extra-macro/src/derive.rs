use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, Ident, LitStr, Variant};

const TITLE_ATTR_IDENT: &str = "title";

pub struct TitledRoutable {
    ident: Ident,
    titles: Vec<LitStr>,
    variants: Punctuated<Variant, syn::token::Comma>,
}

impl Parse for TitledRoutable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let DeriveInput { ident, data, .. } = input.parse()?;

        let data = match data {
            Data::Enum(data) => data,
            Data::Struct(s) => {
                return Err(syn::Error::new(
                    s.struct_token.span(),
                    "expected enum, found struct",
                ))
            }
            Data::Union(u) => {
                return Err(syn::Error::new(
                    u.union_token.span(),
                    "expected enum, found union",
                ))
            }
        };

        let titles = parse_variants_attributes(&data.variants)?;

        Ok(Self {
            ident,
            variants: data.variants,
            titles,
        })
    }
}

fn parse_variants_attributes(
    variants: &Punctuated<Variant, syn::token::Comma>,
) -> syn::Result<Vec<LitStr>> {
    let mut titles: Vec<LitStr> = vec![];

    for variant in variants.iter() {
        if let Fields::Unnamed(ref field) = variant.fields {
            return Err(syn::Error::new(
                field.span(),
                "only named fields are supported",
            ));
        }

        let attrs = &variant.attrs;
        let at_attrs = attrs
            .iter()
            .filter(|attr| attr.path.is_ident(TITLE_ATTR_IDENT))
            .collect::<Vec<_>>();

        let attr = match at_attrs.len() {
            1 => *at_attrs.first().unwrap(),
            0 => {
                return Err(syn::Error::new(
                    variant.span(),
                    format!(
                        "{} attribute must be present on every variant",
                        TITLE_ATTR_IDENT
                    ),
                ))
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    quote! { #(#at_attrs)* },
                    format!("only one {} attribute must be present", TITLE_ATTR_IDENT),
                ))
            }
        };

        let lit = attr.parse_args::<LitStr>()?;
        // let val = lit.value();

        titles.push(lit);
    }

    Ok(titles)
}

impl TitledRoutable {
    fn build_title(&self) -> TokenStream {
        let to_path_matches = self.variants.iter().enumerate().map(|(i, variant)| {
            let ident = &variant.ident;
            let mut right = self.titles.get(i).unwrap().value();

            match &variant.fields {
                Fields::Unit => quote! { Self::#ident => ::std::string::ToString::to_string(#right) },
                Fields::Named(field) => {
                    let fields = field
                        .named
                        .iter()
                        .map(|it| it.ident.as_ref().unwrap())
                        .collect::<Vec<_>>();

                    for field in fields.iter() {
                        // :param -> {param}
                        // *param -> {param}
                        // so we can pass it to `format!("...", param)`
                        right = right.replace(&format!(":{}", field), &format!("{{{}}}", field));
                        right = right.replace(&format!("*{}", field), &format!("{{{}}}", field));
                    }

                    quote! {
                        Self::#ident { #(#fields),* } => ::std::format!(#right, #(#fields = #fields),*)
                    }
                }
                Fields::Unnamed(_) => unreachable!(), // already checked
            }
        });

        quote! {
            fn title(&self) -> ::std::string::String {
                match self {
                    #(#to_path_matches),*,
                }
            }
        }
    }
}

pub fn titled_routable_derive_impl(input: TitledRoutable) -> TokenStream {
    let TitledRoutable { ident, .. } = &input;

    let title = input.build_title();

    quote! {
        #[automatically_derived]
        impl ::yew_router_extra::TitledRoutable for #ident {
            #title
        }
    }
}
