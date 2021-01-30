use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{attr::{FieldAttr, GlobalAttr}, parsers::{FieldPeeker, NamedField, NamedFieldParser, UnnamedField, path_from_ident}};

#[derive(Clone)]
pub struct NamedStructInput {
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub args: Vec<syn::Ident>,
    pub fields: Vec<NamedField>,
}

impl NamedStructInput {
    pub fn new(
        name: syn::Ident,
        generics: syn::Generics,
        attrs: Vec<syn::Attribute>,
        fields: syn::FieldsNamed,
    ) -> Self {
        let args = generics.type_params().cloned().map(|tp| tp.ident).collect();

        let fields = fields
            .named
            .into_iter()
            .map(|field| {
                let syn::Field {
                    ident, attrs, ty, ..
                } = field;
                let attrs = FieldAttr::parse_attrs(attrs);
                NamedField {
                    attrs,
                    name: ident.unwrap(),
                    ty,
                }
            })
            .collect();

        let attrs = GlobalAttr::parse_attrs(attrs);

        NamedStructInput {
            attrs,
            name,
            args,
            fields,
        }
    }

    pub fn process(self) -> NamedStructOutput {
        NamedStructOutput {
            peek_impl: NamedStructPeek::new(self.clone()),
            parse_impl: NamedStructParse::new(self),
        }
    }
}

pub struct NamedStructOutput {
    peek_impl: NamedStructPeek,
    parse_impl: NamedStructParse,
}

impl ToTokens for NamedStructOutput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let NamedStructOutput {
            peek_impl,
            parse_impl,
        } = self;

        peek_impl.to_tokens(tokens);
        parse_impl.to_tokens(tokens);
    }
}

pub struct NamedStructPeek {
    pub fn_impl: TokenStream,
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub where_clause_types: Vec<syn::Type>,
    pub args: Vec<syn::Ident>,
    pub peek_type: syn::Ident,
    pub after_each: TokenStream,
}

impl NamedStructPeek {
    fn new(input: NamedStructInput) -> Self {
        let peek_type = format_ident!("__PeekType");

        let mut peek_impl = NamedStructPeek {
            fn_impl: Default::default(),
            attrs: input.attrs,
            name: input.name,
            where_clause_types: vec![],
            args: input.args,
            peek_type,
            after_each: Default::default(),
        };

        peek_impl.enrich(input.fields);

        peek_impl
    }

    fn enrich(&mut self, fields: Vec<NamedField>) {
        self.fn_impl.extend(
            FieldPeeker {
                attrs: &self.attrs,
                peek_type: &self.peek_type,
                fields: fields
                    .into_iter()
                    .map(|field| {
                        let NamedField { attrs, name: _, ty } = field;
                        UnnamedField { attrs, ty }
                    })
                    .collect(),
            }
            .to_tokens(&mut self.where_clause_types),
        );
    }
}

impl ToTokens for NamedStructPeek {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let NamedStructPeek {
            fn_impl,
            attrs: _,
            after_each: _,
            name,
            where_clause_types,
            args,
            peek_type,
        } = self;

        tokens.extend(quote!{
            #[automatically_derived]
            impl <#peek_type, #(#args),*> ::nommy::Peek<#peek_type> for #name<#(#args),*>
            where #(
                #where_clause_types: ::nommy::Peek<#peek_type>,
            )* {
                fn peek(input: &mut ::nommy::Cursor<impl ::std::iter::Iterator<Item=#peek_type>>) -> bool {
                    #fn_impl
                    true
                }
            }
        })
    }
}

pub struct NamedStructParse {
    pub fn_impl: TokenStream,
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub where_clause_types: Vec<syn::Type>,
    pub args: Vec<syn::Ident>,
    pub parse_type: syn::Ident,
    pub after_each: TokenStream,
}

impl NamedStructParse {
    fn new(input: NamedStructInput) -> Self {
        let parse_type = format_ident!("__ParseType");

        let mut parse_impl = NamedStructParse {
            fn_impl: Default::default(),
            attrs: input.attrs,
            name: input.name,
            where_clause_types: vec![],
            args: input.args,
            parse_type,
            after_each: Default::default(),
        };

        parse_impl.enrich(input.fields);

        parse_impl
    }

    fn enrich(&mut self, fields: Vec<NamedField>) {
        self.fn_impl.extend(
            NamedFieldParser {
                struct_path: path_from_ident(self.name.clone()),
                attrs: &self.attrs,
                parse_type: &self.parse_type,
                fields,
            }
            .to_tokens(&mut self.where_clause_types),
        );
    }
}

impl ToTokens for NamedStructParse {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let NamedStructParse {
            fn_impl,
            attrs: _,
            after_each: _,
            name,
            where_clause_types,
            args,
            parse_type,
        } = self;

        tokens.extend(quote!{
            #[automatically_derived]
            impl <#parse_type, #(#args),*> ::nommy::Parse<#parse_type> for #name<#(#args),*>
            where #(
                #where_clause_types: ::nommy::Parse<#parse_type>,
            )* {
                fn parse(input: &mut ::nommy::Buffer<impl ::std::iter::Iterator<Item=#parse_type>>) -> ::nommy::eyre::Result<Self> {
                    use ::nommy::eyre::WrapErr;
                    #fn_impl
                }
            }
        })
    }
}
