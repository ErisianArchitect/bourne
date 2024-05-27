use syn::parse::discouraged::Speculative;
use syn::parse::*;
use syn::parse_macro_input;

enum Value {
    Null,
    Object(Vec<KeyValuePair>),
    Array(Vec<Value>),
    Expr(syn::Expr),
}

struct Object(Vec<KeyValuePair>);

impl Parse for Object {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        syn::braced!(content in input);
        let tokens = content.parse_terminated(KeyValuePair::parse, syn::Token![,])?;
        let pairs = tokens.into_iter().collect();
        Ok(Object(pairs))
    }
}

struct Array(Vec<Value>);

impl Parse for Array {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        syn::bracketed!(content in input);
        let tokens = content.parse_terminated(Value::parse, syn::Token![,])?;
        let values = tokens.into_iter().collect();
        Ok(Array(values))
    }
}

struct Null;

impl Parse for Null {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        if let Ok(kw) = fork.parse::<syn::Ident>() {
            if kw == "null" {
                input.advance_to(&fork);
                return Ok(Null);
            }
        }
        Err(syn::Error::new(input.span(), "expected `null`"))
    }
}

impl Parse for Value {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(_) = input.parse::<Null>() {
            Ok(Value::Null)
        } else if let Ok(Array(array)) = input.parse::<Array>() {
            Ok(Value::Array(array))
        } else if let Ok(Object(object)) = input.parse::<Object>() {
            Ok(Value::Object(object))
        } else if let Ok(expr) = input.parse::<syn::Expr>() {
            Ok(Value::Expr(expr))
        } else {
            Err(syn::Error::new(input.span(), "Unexpected token."))
        }
    }
}

struct KeyValuePair {
    key: syn::Expr,
    value: Value,
}

impl Parse for KeyValuePair {
    fn parse(input: ParseStream) -> Result<Self> {
        let key = input.parse::<syn::Expr>()?;
        input.parse::<syn::Token![:]>()?;
        let value = input.parse::<Value>()?;
        Ok(KeyValuePair { key, value })
    }
}

impl Value {
    fn to_tokenstream(self) -> proc_macro2::TokenStream {
        use quote::quote;
        match self {
            Value::Null => quote!(bourne::Value::Null),
            Value::Object(object) => {
                let capacity = object.len();
                let inserts = object.into_iter().map(|KeyValuePair { key, value }| {
                    let value = value.to_tokenstream();
                    quote! { map.insert((#key).to_owned(), #value); }
                }).collect::<Vec<_>>();
                quote! {
                    {
                        let mut map = bourne::ValueMap::with_capacity(#capacity);
                        #(#inserts)*
                        bourne::Value::Object(map)
                    }
                }
            },
            Value::Array(array) => {
                let capacity = array.len();
                let lines = array.into_iter().map(|value| {
                    let value = value.to_tokenstream();
                    quote!{ array.push(#value); }
                }).collect::<Vec<_>>();
                quote! {
                    {
                        let mut array = Vec::<bourne::Value>::with_capacity(#capacity);
                        #(#lines)*
                        bourne::Value::Array(array)
                    }
                }
            },
            Value::Expr(expr) => {
                quote!{ bourne::Value::from(#expr) }
            },
        }
    }
}

/// Create a JSON object. Expressions are allowed as values as long as the result is convertible to a Value.
/// Example:
/// ```rust,no_run
/// let number = 3.14;
/// let value = json!(
///     {
///         "number" : number
///     }
/// );
/// ```
#[proc_macro]
pub fn json(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if input.is_empty() {
        quote::quote!{ bourne::Value::Null }.into()
    } else {
        let value = parse_macro_input!(input as Value);
        value.to_tokenstream().into()
    }
}