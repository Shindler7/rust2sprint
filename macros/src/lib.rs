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
        _ => panic!("#[derive(QuoteDisplay)] допустимо использовать только со структурами"),
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
