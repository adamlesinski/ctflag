#![recursion_limit = "256"]

extern crate proc_macro;

mod string_builder;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use string_builder::StringBuilder;
use syn::spanned::Spanned;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
struct Attrs {
    description: Option<String>,
    placeholder: Option<String>,
    default_value: Option<syn::Lit>,
    short_name: Option<syn::LitChar>,
}

enum FlagType {
    Bool,
    Int,
    Float,
    String,
    Option,
    Other,
}

struct Flag {
    name: syn::Ident,
    flag_type: FlagType,
    ty: syn::Type,
    attrs: Attrs,
}

impl Flag {
    fn placeholder(&self) -> Option<&str> {
        self.attrs.placeholder.as_ref().map(String::as_str)
    }

    fn description(&self) -> Option<&str> {
        self.attrs.description.as_ref().map(String::as_str)
    }

    fn default_value(&self) -> Option<&syn::Lit> {
        self.attrs.default_value.as_ref()
    }

    fn short_name(&self) -> Option<&syn::LitChar> {
        self.attrs.short_name.as_ref()
    }
}

#[proc_macro_derive(Flags, attributes(flag))]
pub fn flag_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;
    match collect_flags(&ast) {
        Ok(flags) => {
            if flags.is_empty() {
                return proc_macro::TokenStream::from(
                    quote_spanned! {name.span()=>
                        compiler_error!("Struct must have at least one field.");
                    },
                );
            }

            let temp_vars = flags.iter().map(generate_temp_vars);
            let field_parsing = flags.iter().map(generate_field_parsing);
            let field_assign = flags.iter().map(generate_field_assign);
            let description = generate_description(&flags);
            let expanded = quote! {
                impl ctflag::Flags for #name {
                    fn from_args<T>(args: T) -> ctflag::Result<(Self, Vec<String>)>
                    where T: IntoIterator<Item = String> {
                        #(#temp_vars)*
                        let mut rest_args = Vec::<String>::new();
                        // Skip the first arg (program name) and pass it through.
                        let mut args = args.into_iter();
                        args.next().map(|arg| rest_args.push(arg));
                        let mut iter = ctflag::internal::FlagIterator::from_args(args);
                        while let Some(arg) = iter.next() {
                            use ctflag::internal::Arg;
                            match arg {
                                Arg::Arg(arg) => rest_args.push(arg),
                                Arg::Flag(flag) => {
                                    let arg_name = flag.key;
                                    let arg_value = flag.val;
                                    match arg_name.as_str() {
                                        #(#field_parsing ,)*
                                        _ => {
                                            Err(ctflag::FlagError::UnrecognizedArg(
                                                arg_name))?;
                                        }
                                    }
                                }
                            }
                        }
                        Ok((#name {
                            #(#field_assign),*
                        }, rest_args))
                    }

                    fn description() -> String {
                        #description
                    }
                }
            };
            proc_macro::TokenStream::from(expanded)
        }
        Err(err) => {
            let compile_error = err.to_compile_error();
            proc_macro::TokenStream::from(quote_spanned! {err.span()=>
                #compile_error

                impl ctflag::Flags for #name {
                    fn from_args(args: std::env::Args) -> ctflag::Result<(Self, Vec<String>)> {
                        panic!("Unimplemented");
                    }

                    fn description() -> String {
                        panic!("Unimplemented");
                    }
                }
            })
        }
    }
}

fn generate_temp_vars(flag: &Flag) -> TokenStream {
    let name = &flag.name;
    let name_temp_var =
        syn::Ident::new(&format!("parsed_{}", name), name.span());
    let ty = &flag.ty;
    let rhs = parse_default(flag);
    quote_spanned! {name.span()=>
        let mut #name_temp_var : #ty = #rhs;
    }
}

fn parse_default(flag: &Flag) -> TokenStream {
    if flag.default_value().is_none() {
        return quote_spanned!(flag.ty.span()=> Default::default());
    }
    let default = flag.default_value().unwrap();
    match &flag.flag_type {
        FlagType::Bool | FlagType::Float | FlagType::Int => {
            // Simple assignment will allow the compiler to deal
            // with type mismatches.
            default.clone().into_token_stream()
        }
        FlagType::String => quote!(String::from(#default)),
        FlagType::Option => {
            // Option implies no default.
            quote_spanned! {default.span()=>
                compile_error!("Default value not allowed with Option type")
            }
        }
        FlagType::Other => {
            if let syn::Lit::Str(lit) = default {
                quote! {
                    ctflag::FromArg::from_arg(#lit).expect("failed to parse default flag value")
                }
            } else {
                quote_spanned!(default.span()=> compile_error!("Expected string literal"))
            }
        }
    }
}

fn generate_field_assign(flag: &Flag) -> TokenStream {
    let name = &flag.name;
    let name_temp_var =
        syn::Ident::new(&format!("parsed_{}", name), name.span());
    quote! {
        #name : #name_temp_var
    }
}

fn collect_flags(ast: &syn::DeriveInput) -> syn::Result<Vec<Flag>> {
    if let syn::Data::Struct(struct_data) = &ast.data {
        if let syn::Fields::Named(fields) = &struct_data.fields {
            return fields
                .named
                .iter()
                .map(extract_flag)
                .collect::<syn::Result<Vec<Flag>>>();
        }
    }
    Err(syn::Error::new_spanned(
        ast,
        "Flags can only be derived for named structs",
    ))
}

fn extract_flag(field: &syn::Field) -> syn::Result<Flag> {
    // Parse the attributes into syn::Meta types.
    let attr_metas = field
        .attrs
        .iter()
        .map(syn::Attribute::parse_meta)
        .collect::<syn::Result<Vec<syn::Meta>>>()?;

    let flag_ident = syn::Ident::new("flag", Span::call_site());

    // Find all 'flag' attributes and extract known attributes from them.
    let attrs: Attrs = attr_metas
        .iter()
        .filter(|m| m.name() == flag_ident)
        .map(extract_flag_attrs)
        .next()
        .unwrap_or_else(|| Ok(Attrs::default()))?;

    Ok(Flag {
        name: field.ident.as_ref().unwrap().clone(),
        flag_type: extract_flag_type(&field.ty),
        ty: field.ty.clone(),
        attrs: attrs,
    })
}

fn extract_flag_type(ty: &syn::Type) -> FlagType {
    if is_bool(ty) {
        FlagType::Bool
    } else if is_int(ty) {
        FlagType::Int
    } else if is_float(ty) {
        FlagType::Float
    } else if is_string(ty) {
        FlagType::String
    } else if let Some(_) = extract_option_param_type(ty) {
        FlagType::Option
    } else {
        FlagType::Other
    }
}

fn is_bool(ty: &syn::Type) -> bool {
    if let syn::Type::Path(p) = ty {
        p.path.is_ident(syn::Ident::new("bool", Span::call_site()))
    } else {
        false
    }
}

fn is_int(ty: &syn::Type) -> bool {
    if let syn::Type::Path(p) = ty {
        [
            "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "u64",
            "usize",
        ]
        .iter()
        .filter(|ident| {
            p.path.is_ident(syn::Ident::new(ident, Span::call_site()))
        })
        .next()
        .is_some()
    } else {
        false
    }
}

fn is_float(ty: &syn::Type) -> bool {
    if let syn::Type::Path(p) = ty {
        ["f32", "f64"]
            .iter()
            .filter(|ident| {
                p.path.is_ident(syn::Ident::new(ident, Span::call_site()))
            })
            .next()
            .is_some()
    } else {
        false
    }
}

fn is_string(ty: &syn::Type) -> bool {
    if let syn::Type::Path(p) = ty {
        p.path
            .is_ident(syn::Ident::new("String", Span::call_site()))
    } else {
        false
    }
}

fn extract_option_param_type(ty: &syn::Type) -> Option<syn::Type> {
    if let syn::Type::Path(p) = ty {
        if p.path.leading_colon.is_none()
            && p.path.segments.len() == 1
            && p.path.segments[0].ident
                == syn::Ident::new("Option", Span::call_site())
        {
            if let syn::PathArguments::AngleBracketed(args) =
                &p.path.segments[0].arguments
            {
                if let syn::GenericArgument::Type(ty) =
                    &args.args.iter().next()?
                {
                    return Some(ty.clone());
                }
            }
        }
    }
    None
}

fn extract_flag_attrs(meta: &syn::Meta) -> syn::Result<Attrs> {
    let mut attrs = Attrs::default();
    if let syn::Meta::List(l) = meta {
        for nested in &l.nested {
            if let syn::NestedMeta::Meta(syn::Meta::NameValue(name_val)) =
                nested
            {
                if name_val.ident == syn::Ident::new("desc", Span::call_site())
                {
                    attrs.description =
                        Some(parse_flag_attr_description(&name_val.lit)?);
                } else if name_val.ident
                    == syn::Ident::new("placeholder", Span::call_site())
                {
                    attrs.placeholder =
                        Some(parse_flag_attr_placeholder(&name_val.lit)?);
                } else if name_val.ident
                    == syn::Ident::new("default", Span::call_site())
                {
                    attrs.default_value = Some(name_val.lit.clone());
                } else if name_val.ident
                    == syn::Ident::new("short", Span::call_site())
                {
                    attrs.short_name =
                        Some(parse_flag_attr_short_name(&name_val.lit)?);
                } else {
                    return Err(syn::Error::new_spanned(
                        &name_val.ident,
                        format!("Unknown flags attribute '{}'", name_val.ident),
                    ));
                }
            } else {
                return Err(syn::Error::new_spanned(
                    &l.nested,
                    "Unexpected attribute syntax",
                ));
            }
        }
    } else {
        return Err(syn::Error::new_spanned(
            meta,
            "Unexpected attribute syntax",
        ));
    }
    return Ok(attrs);
}

fn parse_flag_attr_description(literal: &syn::Lit) -> syn::Result<String> {
    if let syn::Lit::Str(desc) = literal {
        Ok(desc.value())
    } else {
        Err(syn::Error::new_spanned(
            literal,
            "Description must be a string literal",
        ))
    }
}

fn parse_flag_attr_placeholder(literal: &syn::Lit) -> syn::Result<String> {
    if let syn::Lit::Str(val) = literal {
        Ok(val.value())
    } else {
        Err(syn::Error::new_spanned(
            literal,
            "Placeholder must be a string literal",
        ))
    }
}

fn parse_flag_attr_short_name(literal: &syn::Lit) -> syn::Result<syn::LitChar> {
    if let syn::Lit::Char(val) = literal {
        Ok(val.clone())
    } else {
        Err(syn::Error::new_spanned(
            literal,
            "Short name must be a char literal",
        ))
    }
}

fn generate_field_parsing(flag: &Flag) -> TokenStream {
    let name = &flag.name;
    let name_lit = name.to_string();
    let name_temp_var =
        syn::Ident::new(&format!("parsed_{}", name), name.span());
    let ty = &flag.ty;
    let parse_expr = match &flag.flag_type {
        FlagType::Bool => {
            quote_spanned! {name.span()=>
                ctflag::internal::bool_from_arg(arg_value.as_ref().map(|s| s.as_str()))
                    .map_err(|err| ctflag::FlagError::ParseError(
                        ctflag::ParseErrorStruct {
                            type_str: "bool",
                            input: arg_value.unwrap(),
                            src: err,
                        }
                    ))?
            }
        }
        FlagType::Option => quote_spanned! {name.span()=> {
            let input = arg_value
                .or_else(|| iter.next_arg())
                .ok_or(ctflag::FlagError::MissingValue(String::from(#name_lit)))?;
            ctflag::internal::option_from_arg(&input)
                .map_err(|err| ctflag::FlagError::ParseError(
                    ctflag::ParseErrorStruct {
                        type_str: stringify!(#ty),
                        input: input,
                        src: err,
                    }
                ))?
            }
        },
        _ => quote_spanned! {name.span()=> {
            let input = arg_value
                .or_else(|| iter.next_arg())
                .ok_or(ctflag::FlagError::MissingValue(String::from(#name_lit)))?;
            ctflag::FromArg::from_arg(&input)
                .map_err(|err| ctflag::FlagError::ParseError(
                    ctflag::ParseErrorStruct {
                        type_str: stringify!(#ty),
                        input: input,
                        src: err,
                    }
                ))?
            }
        },
    };

    let long_name = syn::LitStr::new(&format!("--{}", name), name.span());
    let match_case = if let Some(short_name) = flag.short_name() {
        let short_name = syn::LitStr::new(
            &format!("-{}", &short_name.value().to_string()),
            short_name.span(),
        );
        quote! {
            #long_name | #short_name
        }
    } else {
        long_name.into_token_stream()
    };

    quote_spanned! {name.span()=>
        #match_case => {
            #name_temp_var = #parse_expr ;
        }
    }
}

fn generate_flag_name_and_len(flag: &Flag) -> (String, usize) {
    let mut buf = StringBuilder::new();
    if let Some(short_name) = flag.short_name() {
        buf.append(&format!("-{}, ", short_name.value()));
    } else {
        buf.append("    ");
    }
    buf.append(&format!("--{}", &flag.name));
    match &flag.flag_type {
        FlagType::Bool => {}
        FlagType::Option => {
            buf.append(&format!(" [{}]", flag.placeholder().unwrap_or("VALUE")))
        }
        _ => buf.append(&format!(" {}", flag.placeholder().unwrap_or("VALUE"))),
    };
    let s = String::from(buf);
    let len = s.as_str().graphemes(true).count();
    (s, len)
}

fn generate_description(flag: &[Flag]) -> TokenStream {
    let flag_names = flag
        .iter()
        .map(generate_flag_name_and_len)
        .collect::<Vec<(String, usize)>>();
    let col_width = flag_names.iter().map(|(_, len)| len).max().unwrap();

    let mut buf = StringBuilder::new();
    buf.append("OPTIONS:\n");

    for ((flag_name, len), flag) in flag_names.iter().zip(flag.iter()) {
        buf.append("  ");
        buf.append(flag_name);
        if flag.description().is_some() || flag.default_value().is_some() {
            buf.append("    ");
            for _ in 0..(col_width - len) {
                buf.append(" ");
            }
            if let Some(desc) = flag.description() {
                buf.append(&desc);
            }
            if let Some(def) = flag.default_value() {
                use quote::ToTokens;
                buf.append(&format!(
                    " (defaults to {})",
                    &def.clone().into_token_stream()
                ));
            }
        }
        buf.append("\n");
    }
    let desc = String::from(buf);
    quote! {
        String::from(#desc)
    }
}
