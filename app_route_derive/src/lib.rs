#![recursion_limit = "256"]

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
use regex::Regex;
use std::collections::HashSet;
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, PartialEq)]
enum PathToRegexError {
	MissingLeadingForwardSlash,
	NonAsciiChars,
	InvalidIdentifier(String),
	InvalidTrailingSlash,
}

fn path_to_regex(path: &str) -> Result<(String, String), PathToRegexError> {
	enum ParseState {
		Initial,
		Static,
		VarName(String),
	};

	if !path.is_ascii() {
		return Err(PathToRegexError::NonAsciiChars);
	}

	let ident_regex = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").unwrap();

	let mut regex = "".to_string();
	let mut format_str = "".to_string();
	let mut parse_state = ParseState::Initial;

	for byte in path.chars() {
		match parse_state {
			ParseState::Initial => {
				if byte != '/' {
					return Err(PathToRegexError::MissingLeadingForwardSlash);
				}

				regex += "^/";
				format_str += "/";

				parse_state = ParseState::Static;
			}
			ParseState::Static => {
				if byte == ':' {
					format_str.push('{');
					parse_state = ParseState::VarName("".to_string());
				} else {
					regex.push(byte);
					format_str.push(byte);
					parse_state = ParseState::Static;
				}
			}
			ParseState::VarName(mut name) => {
				if byte == '/' {
					// Validate 'name' as a Rust identifier
					if !ident_regex.is_match(&name) {
						return Err(PathToRegexError::InvalidIdentifier(name));
					}

					format_str += &format!("{}}}/", name);
					regex += &format!("(?P<{}>[^/]+)/", name);
					parse_state = ParseState::Static;
				} else {
					name.push(byte);
					parse_state = ParseState::VarName(name);
				}
			}
		};
	}

	if let ParseState::VarName(name) = parse_state {
		regex += &format!("(?P<{}>[^/]+)", name);
		format_str += &format!("{}}}", name);
	}

	if regex.ends_with('/') {
		return Err(PathToRegexError::InvalidTrailingSlash);
	}

	regex += "$";

	Ok((regex, format_str))
}

#[test]
fn test_path_to_regex() {
	let (regex, _) = path_to_regex("/p/:project_id/exams/:exam_id/submissions_expired").unwrap();
	assert_eq!(
		regex,
		r"^/p/(?P<project_id>[^/]+)/exams/(?P<exam_id>[^/]+)/submissions_expired$"
	);
}

#[test]
fn test_path_to_regex_no_path_params() {
	let (regex, _) = path_to_regex("/p/exams/submissions_expired").unwrap();
	assert_eq!(regex, r"^/p/exams/submissions_expired$");
}

#[test]
fn test_path_to_regex_no_leading_slash() {
	let regex = path_to_regex("p/exams/submissions_expired");
	assert_eq!(regex, Err(PathToRegexError::MissingLeadingForwardSlash));
}

#[test]
fn test_path_to_regex_non_ascii_chars() {
	let regex = path_to_regex("🥖p🥖:project_id🥖exams🥖:exam_id🥖submissions_expired");
	assert_eq!(regex, Err(PathToRegexError::NonAsciiChars));
}

#[test]
fn test_path_to_regex_invalid_ident() {
	let regex = path_to_regex("/p/:project_id/exams/:exam*ID/submissions_expired");
	assert_eq!(
		regex,
		Err(PathToRegexError::InvalidIdentifier("exam*ID".to_string()))
	);

	let regex = path_to_regex("/p/:project_id/exams/:_exam_id/submissions_expired");
	assert_eq!(
		regex,
		Err(PathToRegexError::InvalidIdentifier("_exam_id".to_string()))
	);
}

#[test]
fn test_path_to_regex_invalid_ending() {
	let regex = path_to_regex("/p/:project_id/exams/:exam_id/submissions_expired/");
	assert_eq!(regex, Err(PathToRegexError::InvalidTrailingSlash));
}

fn get_string_attr(name: &str, attrs: &[syn::Attribute]) -> Option<String> {
	for attr in attrs {
		let attr = attr.parse_meta();

		if let Ok(syn::Meta::List(ref list)) = attr {
			if list.ident == name {
				for thing in &list.nested {
					if let syn::NestedMeta::Literal(syn::Lit::Str(str_lit)) = thing {
						return Some(str_lit.value());
					}
				}
			}
		}
	}

	None
}

fn has_flag_attr(name: &str, attrs: &[syn::Attribute]) -> bool {
	for attr in attrs {
		let attr = attr.parse_meta();

		if let Ok(syn::Meta::Word(ref ident)) = attr {
			if ident == name {
				return true;
			}
		}
	}

	false
}

fn get_struct_fields(data: &syn::Data) -> Vec<syn::Field> {
	match data {
		syn::Data::Struct(data_struct) => match data_struct.fields {
			syn::Fields::Named(ref named_fields) => named_fields.named.iter().cloned().collect(),
			_ => panic!("Struct fields must be named"),
		},
		_ => panic!("AppRoute derive is only supported for structs"),
	}
}

fn field_is_option(field: &syn::Field) -> bool {
	match field.ty {
		syn::Type::Path(ref type_path) => type_path
			.path
			.segments
			.iter()
			.last()
			.map(|segment| segment.ident == "Option")
			.unwrap_or(false),
		_ => false,
	}
}

#[proc_macro_derive(AppRoute, attributes(path, query))]
pub fn app_path_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let struct_fields = get_struct_fields(&input.data);

	let (path_fields, query_fields): (Vec<_>, Vec<_>) = struct_fields
		.into_iter()
		.partition(|f| !has_flag_attr("query", &f.attrs));

	let name = &input.ident;
	let generics = input.generics;
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let path_string = get_string_attr("path", &input.attrs);

	let url_path = path_string
		.expect("derive(AppRoute) requires a #[path(\"/your/path/here\")] attribute on the struct");

	let (path_regex_str, format_str) =
		path_to_regex(&url_path).expect("Could not convert path attribute to a valid regex");

	// Validate path_regex and make sure struct and path have matching fields
	let path_regex =
		Regex::new(&path_regex_str).expect("path attribute was not compiled into a valid regex");

	let regex_capture_names_set: HashSet<String> = path_regex
		.capture_names()
		.filter_map(|c_opt| c_opt.map(|c| c.to_string()))
		.collect();
	let field_names_set: HashSet<String> = path_fields
		.clone()
		.into_iter()
		.map(|f| f.ident.unwrap().to_string())
		.collect();

	if regex_capture_names_set != field_names_set {
		let missing_from_path = field_names_set.difference(&regex_capture_names_set);
		let missing_from_struct = regex_capture_names_set.difference(&field_names_set);

		let error_msg = format!("\nFields in struct missing from path pattern: {:?}\nFields in path missing from struct: {:?}", missing_from_path, missing_from_struct);
		panic!(error_msg);
	}

	let path_field_assignments = path_fields.clone().into_iter().map(|f| {
		let f_ident = f.ident.unwrap();
		let f_ident_str = f_ident.to_string();

		quote! {
			#f_ident: captures[#f_ident_str].parse().map_err(|e| {
				RouteParseErr::ParamParseErr(std::string::ToString::to_string(&e))
			})?
		}
	});

	let query_field_assignments = query_fields.clone().into_iter().map(|f| {
        let is_option = field_is_option(&f);
        let f_ident = f.ident.unwrap();

        if is_option {
            quote! {
                #f_ident: query_string.and_then(|q| qs::from_str(q).ok())
            }
        } else {
            quote! {
                #f_ident: qs::from_str(query_string.ok_or(RouteParseErr::NoQueryString)?).map_err(|e| RouteParseErr::QueryParseErr(e.description().to_string()))?
            }
        }
    });

	let path_field_parsers = quote! {
		#(
			#path_field_assignments
		),*
	};

	let query_field_parsers = quote! {
		#(
			#query_field_assignments
		),*
	};

	let format_args = path_fields.clone().into_iter().map(|f| {
		let f_ident = f.ident.unwrap();

		quote! {
			#f_ident = self.#f_ident
		}
	});

	let format_args = quote! {
		#(
			#format_args
		),*
	};

	let query_field_to_string_statements = query_fields.into_iter().map(|f| {
		let is_option = field_is_option(&f);
		let f_ident = f.ident.unwrap();

		if is_option {
			quote! {
				self.#f_ident.as_ref().and_then(|q| qs::to_string(&q).ok())
			}
		} else {
			quote! {
				qs::to_string(&self.#f_ident).ok()
			}
		}
	});

	let encoded_query_fields = quote! {
		#(
			#query_field_to_string_statements
		),*
	};

	let struct_constructor = match (
		path_field_parsers.is_empty(),
		query_field_parsers.is_empty(),
	) {
		(true, true) => quote! {
			#name {}
		},
		(true, false) => quote! {
			#name {
				#query_field_parsers
			}
		},
		(false, true) => quote! {
			#name {
				#path_field_parsers
			}
		},
		(false, false) => quote! {
			#name {
				#path_field_parsers,
				#query_field_parsers
			}
		},
	};

	let app_path_impl = quote! {
		impl #impl_generics app_route::AppRoute for #name #ty_generics #where_clause {

			fn path_pattern() -> String {
				#path_regex_str.to_string()
			}

			fn query_string(&self) -> Option<String> {
				use app_route::serde_qs as qs;

				// TODO - Remove duplicates because
				//        there could be multiple fields with
				//        a #[query] attribute that have common fields

				// TODO - can this be done with an on-stack array?
				let encoded_queries: Vec<Option<String>> = vec![#encoded_query_fields];
				let filtered: Vec<_> = encoded_queries.into_iter().filter_map(std::convert::identity).collect();

				if !filtered.is_empty() {
					Some(filtered.join("&"))
				} else {
					None
				}
			}
		}

		impl #impl_generics std::fmt::Display for #name #ty_generics #where_clause {
			fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				if let Some(query) = self.query_string() {
					let path = format!(
						#format_str,
						#format_args
					);

					write!(f, "{}?{}", path, query)
				} else {
					write!(
						f,
						#format_str,
						#format_args
					)
				}
			}
		}

		impl #impl_generics std::str::FromStr for #name #ty_generics #where_clause {
			type Err = app_route::RouteParseErr;

			fn from_str(app_path: &str) -> Result<Self, Self::Err> {
				use app_route::serde_qs as qs;
				use app_route::RouteParseErr;

				app_route::lazy_static! {
					static ref PATH_REGEX: app_route::Regex = app_route::Regex::new(#path_regex_str).expect("Failed to compile regex");
				}

				let question_pos = app_path.find('?');
				let just_path = &app_path[..(question_pos.unwrap_or_else(|| app_path.len()))];

				let captures = (*PATH_REGEX).captures(just_path).ok_or(RouteParseErr::NoMatches)?;

				let query_string = question_pos.map(|question_pos| {
					let mut query_string = &app_path[question_pos..];

					if query_string.starts_with('?') {
						query_string = &query_string[1..];
					}

					query_string
				});

				Ok(#struct_constructor)
			}
		}
	};

	let impl_wrapper = syn::Ident::new(
		&format!("_IMPL_APPROUTE_FOR_{}", name.to_string()),
		proc_macro2::Span::call_site(),
	);

	let out = quote! {
		const #impl_wrapper: () = {
			extern crate app_route;
			#app_path_impl
		};
	};

	out.into()
}
