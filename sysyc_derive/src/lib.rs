use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
	parse2, parse_macro_input, Data, DeriveInput, Field, FieldMutability, Fields,
	Ident, Visibility, parse_quote,
};

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
		attrs: Vec::new(), // TODO
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
		}
	}
	.into()
}
