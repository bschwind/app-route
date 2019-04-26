#[doc(hidden)]
pub use lazy_static::lazy_static;

#[doc(hidden)]
pub use regex::Regex;

#[doc(hidden)]
pub use serde_qs;

pub use app_route_derive::AppRoute;

#[derive(Debug)]
pub enum RouteParseErr {
	NoMatches,
	NoQueryString,
	ParamParseErr(String),
	QueryParseErr(String),
}

pub trait AppRoute: std::fmt::Display + std::str::FromStr {
	fn path_pattern() -> String
	where
		Self: Sized;
	fn query_string(&self) -> Option<String>;
}
