use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::*;

#[proc_macro_attribute]
pub fn has_attrs(_: TokenStream, item: TokenStream) -> TokenStream {
	let mut input = parse_macro_input!(item as DeriveInput);

	let Data::Struct(data) = &mut input.data else {
		panic!("This trait can only implement on struct");
	};

	let Fields::Named(fields) = &mut data.fields else {
		panic!("made, struct li mian zen me neng shi mei name de");
	};

	fields.named.push(Field {
		attrs: Vec::new(),
		vis: Visibility::Public(parse_quote!(pub)),
		mutability: FieldMutability::None,
		ident: Some(Ident::new("_attrs", Span::call_site())),
		colon_token: None,
		ty: parse2(quote!(::std::collections::HashMap<String, Attr>)).unwrap(),
	});

	let name = &input.ident;

	quote! {
		#input

		impl Attrs for #name {
			fn set_attr(&mut self, name: &str, attr: Attr) {
				self._attrs.insert(String::from(name), attr);
			}

			fn get_attr(&self, name: &str) -> Option<&Attr> {
				self._attrs.get(name)
			}

			fn get_name(&self) -> String {
				stringify!(#name).to_string()
			}
		}
	}
	.into()
}

#[proc_macro_derive(AstNode)]
pub fn ast_node_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = input.ident;
	let snake_name = name.to_string().to_snake_case();
	let visitor_fn_name = format!("visit_{}", snake_name);
	let visitor_fn_ident = syn::Ident::new(&visitor_fn_name, name.span());

	let expanded = quote! {
		impl AstNode for #name {
			fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<()> {
				visitor.#visitor_fn_ident(self)
			}
		}
	};

	TokenStream::from(expanded)
}

#[proc_macro_derive(Fuyuki, attributes(style))]
pub fn display_lowercase(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = input.ident;
	if let Data::Enum(DataEnum { variants, .. }) = input.data {
		let cases = variants.into_iter().map(|v| {
			let variant_name = &v.ident;
			let variant_str = variant_name.to_string().to_lowercase();
			for attr in &v.attrs {
				if attr.path().is_ident("style") {
					if let Ok(lit_str) = attr.parse_args::<LitStr>() {
						let val = lit_str.value();
						return {
							quote! {
								#name::#variant_name => write!(f, "{}", #val)
							}
						};
					} else {
						return Error::new_spanned(attr, "Expected a string literal")
							.to_compile_error();
					}
				}
			}
			quote! {
				#name::#variant_name => write!(f, "{}", #variant_str)
			}
		});

		let expanded = quote! {
			impl std::fmt::Display for #name {
				fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
					match self {
						#( #cases, )*
					}
				}
			}
		};

		TokenStream::from(expanded)
	} else {
		panic!("DisplayLowercase is only defined for enums");
	}
}
