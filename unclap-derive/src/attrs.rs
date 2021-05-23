use darling::{
    util::{Override, SpannedValue},
    FromField, FromMeta, FromVariant,
};
use syn::LitStr;

#[derive(FromMeta)]
#[darling(default)]
pub enum FieldVariant {
    #[darling(rename = "named")]
    Named(Override<LitStr>),
    #[darling(rename = "unnamed")]
    Unnamed(Override<()>),
    #[darling(rename = "flag")]
    Flag(Override<LitStr>),
}

impl Default for FieldVariant {
    fn default() -> Self {
        FieldVariant::Unnamed(Override::Inherit)
    }
}

#[derive(FromField)]
#[darling(attributes(argument))]
pub struct UnclapField {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
    #[darling(default)]
    pub variant: SpannedValue<FieldVariant>,
}

#[derive(FromMeta)]
#[darling(default)]
pub enum EnumVariant {
    #[darling(rename = "named")]
    Named(Override<LitStr>),
    #[darling(rename = "unnamed")]
    Unnamed(Override<()>),
}

impl Default for EnumVariant {
    fn default() -> Self {
        EnumVariant::Unnamed(Override::Inherit)
    }
}

#[derive(FromVariant)]
#[darling(attributes(argument))]
pub struct UnclapVariant_ {
    pub ident: syn::Ident,
    pub fields: darling::ast::Fields<syn::Field>,
    #[darling(default)]
    pub variant: SpannedValue<EnumVariant>,
}
pub type UnclapVariant = SpannedValue<UnclapVariant_>;
