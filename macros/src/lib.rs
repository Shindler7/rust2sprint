//! Набор универсальных макросов для приложений Quote.
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, LitStr};

/// Макрос `QuoteDisplay` автоматически генерирует для структуры реализацию
/// `Display` и `FromStr`, чтобы сериализовать/десериализовать её в строковый
/// формат с полями, разделёнными |. Он валидирует количество полей и
/// возвращает понятные ошибки парсинга.
#[proc_macro_derive(QuoteDisplay)]
pub fn macros_quote_display(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Сбор названия полей.
    let fields_name = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("QuoteDisplay допустимо использовать только со структурами"),
    };

    let fields: Vec<_> = fields_name
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .collect();

    let fields_count = fields.len();

    // Формат для Display: "{}|{}|{}"
    let fmt_string = vec!["{}"; fields_count].join("|");
    let fmt_lit = LitStr::new(&fmt_string, proc_macro2::Span::call_site());

    let fields_parses: Vec<_> = (0..fields_count)
        .map(|i| {
            let field_name = &fields[i];
            quote! {
                #field_name: parts[#i]
                    .parse()
                    .map_err(|_| QuoteError::value_err(format!(
                        "Ошибка парсинга строки {} на позиции {} для поля {}",
                        s, #i, stringify!(#field_name)
                    )))?,
            }
        })
        .collect();

    let output: proc_macro2::TokenStream = {
        quote! {
            impl std::fmt::Display for #struct_name {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    writeln!(
                        f,
                        #fmt_lit,
                        #(self.#fields),*
                    )
                }
            }

            impl std::str::FromStr for #struct_name {
                type Err = QuoteError;
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    let parts: Vec<&str> = s.split('|').collect();
                    if parts.len() != #fields_count {
                        Err(QuoteError::value_err(format!(
                            "ожидается {} типа, разделённых '|', получено {} в строке {}",
                            #fields_count,
                            parts.len(),
                            s
                        )))
                    } else {
                        Ok(Self {
                            #(#fields_parses)*
                        })
                    }
                }
            }
        }
    };

    TokenStream::from(output)
}

/// Derive-макрос для `Enum`: автоматически добавляет реализации
/// [`std::fmt::Display`] и [`std::str::FromStr`].
///
/// ## Пример
///
/// ```ignore
/// use macros::QuoteEnumDisplay;
///
/// #[derive(Debug, Clone, QuoteEnumDisplay)]
/// enum Commands {
///     #[str("start")]
///     Start,
///     #[str("stop")]
///     Stop,
/// }
///
/// let start = Commands::Start;
/// assert_eq!(stringify!(start), "start");
/// ```
#[proc_macro_derive(QuoteEnumDisplay, attributes(str))]
pub fn derive_display_fromstr(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = match input.data {
        Data::Enum(e) => e.variants,
        _ => panic!("QuoteEnumDisplay допустимо использовать только с enum"),
    };

    let mut to_arms = Vec::new();
    let mut from_arms = Vec::new();

    for v in variants {
        let ident = v.ident;
        if !matches!(v.fields, Fields::Unit) {
            panic!("Только unit-variants");
        }
        let mut lit = ident.to_string().to_lowercase();
        for attr in v.attrs {
            if attr.path().is_ident("str") {
                let s: syn::LitStr = attr.parse_args().expect("str(\"...\")");
                lit = s.value();
            }
        }
        let lit_str = syn::LitStr::new(&lit, proc_macro2::Span::call_site());
        to_arms.push(quote! { #name::#ident => write!(f, #lit_str), });
        from_arms.push(quote! { #lit_str => Ok(#name::#ident), });
    }

    let expanded = quote! {
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self { #(#to_arms)* }
            }
        }

        impl std::str::FromStr for #name {
            type Err = QuoteError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.trim().to_lowercase().as_str() {
                    #(#from_arms)*
                    _ => Err(QuoteError::value_err(format!(
                        "Некорректное значение {}: {}",
                        stringify!(#name), s
                    ))),
                }
            }
        }
    };

    TokenStream::from(expanded)
}
