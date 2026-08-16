extern crate proc_macro;

use convert_case::{Case, Casing};
use darling::{FromDeriveInput, FromField, ast::Data, util::Ignored};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{Attribute, DeriveInput, Generics, parse_macro_input};

#[derive(snafu::Snafu, Debug)]
enum GeneratorError {
  #[snafu(transparent)]
  Syn { source: syn::Error },

  #[snafu(transparent)]
  Darling { source: darling::Error },
}

impl GeneratorError {
  fn write_errors(self) -> proc_macro2::TokenStream {
    match self {
      GeneratorError::Syn { source } => source.to_compile_error(),
      GeneratorError::Darling { source } => source.write_errors(),
    }
  }
}

#[derive(Debug, FromField)]
#[darling(attributes(dyngql))]
#[allow(dead_code)]
struct DynamicGraphqlFieldInfo {
  ident: Option<Ident>,
  ty: syn::Type,
}

#[derive(FromDeriveInput)]
#[darling(attributes(dyngql), forward_attrs(doc))]
#[allow(dead_code)]
struct DynamicGraphqlInfo {
  pub ident: Ident,
  pub attrs: Vec<Attribute>,
  pub generics: Generics,
  pub data: Data<Ignored, DynamicGraphqlFieldInfo>,
}

impl DynamicGraphqlInfo {
  fn expand(&self) -> Result<TokenStream, GeneratorError> {
    let struct_name = &self.ident;
    let enum_name = format_ident!("{}FieldEnum", struct_name);

    let fields = self.data.as_ref().take_struct().unwrap();

    let enum_variants = fields
      .iter()
      .filter_map(|field| field.ident.as_ref())
      .map(|field_ident| {
        let variant_name = Ident::new(&field_ident.to_string().to_case(Case::Pascal), field_ident.span());
        quote! { #variant_name }
      })
      .collect::<Vec<_>>();

    let as_str_arms: Vec<_> = fields
      .iter()
      .filter_map(|field| field.ident.as_ref())
      .map(|field_ident| {
        let variant_name = Ident::new(&field_ident.to_string().to_case(Case::Pascal), field_ident.span());
        let field_name_str = field_ident.to_string().to_case(Case::Camel);
        quote! {
            Self::#variant_name => #field_name_str,
        }
      })
      .collect::<Vec<_>>();

    let from_str_arms: Vec<_> = fields
      .iter()
      .filter_map(|field| field.ident.as_ref())
      .map(|field_ident| {
        let variant_name = Ident::new(&field_ident.to_string().to_case(Case::Pascal), field_ident.span());
        let field_name_str = field_ident.to_string().to_case(Case::Camel);
        quote! {
            #field_name_str => Some(Self::#variant_name)
        }
      })
      .collect();

    let all_field_names: Vec<_> = fields
      .iter()
      .filter_map(|field| field.ident.as_ref())
      .map(|field_ident| field_ident.to_string().to_case(Case::Camel))
      .collect();

    let result = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum #enum_name {
            #(#enum_variants),*
        }

        impl #enum_name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    #(#as_str_arms),*
                }
            }

            pub fn from_str(s: &str) -> Option<Self> {
                match s {
                    #(#from_str_arms),* ,
                    _ => None
                }
            }

            pub fn all_field_names() -> Vec<&'static str> {
                vec![#(#all_field_names),*]
            }
        }

        impl std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl From<#enum_name> for String {
            fn from(value: #enum_name) -> Self {
                value.as_str().to_string()
            }
        }

        impl std::str::FromStr for #enum_name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::from_str(s).ok_or_else(|| format!("Unknown field name: {s}"))
            }
        }

    };

    Ok(result.into())
  }
}

#[proc_macro_derive(DynamicGraphql, attributes(dyngql))]
pub fn derive_dynamic_graphql(input: TokenStream) -> TokenStream {
  let opts = match DynamicGraphqlInfo::from_derive_input(&parse_macro_input!(input as DeriveInput)) {
    Ok(opts) => opts,
    Err(err) => return TokenStream::from(err.write_errors()),
  };
  match opts.expand() {
    Ok(token_stream) => token_stream,
    Err(err) => err.write_errors().into(),
  }
}
