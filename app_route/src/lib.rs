/*!
# Usage

`src/Cargo.toml`
```toml
[dependencies]
app_route = "0.1"
serde = { version = "1.0", features = ["derive"] }
```

`main.rs`
```rust
use app_route::AppRoute;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct UserListQuery {
    limit: Option<u64>,
    offset: Option<u64>,
    keyword: Option<String>,

    #[serde(default)]
    friends_only: bool,
}

#[derive(AppRoute, Debug, PartialEq)]
#[path("/groups/:group_id/users")]
struct UsersListRoute {
    group_id: u64,

    #[query]
    query: UserListQuery,
}

fn main() {
    let path: UsersListRoute =
        "/groups/4313145/users?offset=10&limit=20&friends_only=false&keyword=some_keyword"
            .parse()
            .unwrap();

    assert_eq!(
        path,
        UsersListRoute {
            group_id: 4313145,
            query: {
                UserListQuery {
                    limit: Some(20),
                    offset: Some(10),
                    keyword: Some("some_keyword".to_string()),
                    friends_only: false,
                }
            }
        }
    );

    println!("Path: {}", path);
    // Output:
    // Path: /groups/4313145/users?limit=20&offset=10&keyword=some_keyword&friends_only=false
}
```
*/

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
