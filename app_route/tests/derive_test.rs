use app_route::{AppRoute, RouteParseErr};
use serde::{Deserialize, Serialize};

#[derive(AppRoute, Debug, PartialEq)]
#[route("/users")]
struct UsersListPath {}

#[test]
fn no_params() {
	let path: UsersListPath = "/users".parse().unwrap();
	assert_eq!(path, UsersListPath {});
}

#[test]
fn trailing_slash() {
	let path: Result<UsersListPath, _> = "/users/".parse();
	match path {
		Err(RouteParseErr::NoMatches) => {}
		_ => assert!(false),
	}
}

#[test]
fn no_leading_slash() {
	let path: Result<UsersListPath, _> = "users".parse();
	match path {
		Err(RouteParseErr::NoMatches) => {}
		_ => assert!(false),
	}
}

#[derive(AppRoute, Debug, PartialEq)]
#[route("/users/:user_id")]
struct UserDetailPath {
	user_id: u64,
}

#[test]
fn one_param() {
	let path: UserDetailPath = "/users/642151".parse().unwrap();
	assert_eq!(path, UserDetailPath { user_id: 642151 });
}

#[test]
fn invalid_param_type() {
	let path: Result<UserDetailPath, _> = "/users/not_a_u64".parse();
	match path {
		Err(RouteParseErr::ParamParseErr(_)) => {}
		_ => assert!(false),
	}
}

#[test]
fn one_param_no_leading_slash() {
	let path: Result<UserDetailPath, _> = "users/4216".parse();
	match path {
		Err(RouteParseErr::NoMatches) => {}
		_ => assert!(false),
	}
}

#[derive(AppRoute, Debug, PartialEq)]
#[route("/users/:user_id/friends/:friend_name")]
struct UserFriendDetailPath {
	user_id: u64,
	friend_name: String,
}

#[test]
fn two_params() {
	let path: UserFriendDetailPath = "/users/612451/friends/steve".parse().unwrap();
	assert_eq!(
		path,
		UserFriendDetailPath {
			user_id: 612451,
			friend_name: "steve".to_string()
		}
	);
}

#[test]
fn two_params_utf8_1() {
	let path: UserFriendDetailPath = "/users/612451/friends/田中".parse().unwrap();
	assert_eq!(
		path,
		UserFriendDetailPath {
			user_id: 612451,
			friend_name: "田中".to_string()
		}
	);
}

#[test]
fn two_params_utf8_2() {
	let path: UserFriendDetailPath = "/users/612451/friends/🌮🌮🌮".parse().unwrap();
	assert_eq!(
		path,
		UserFriendDetailPath {
			user_id: 612451,
			friend_name: "🌮🌮🌮".to_string()
		}
	);
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct UserListQuery {
	limit: Option<u64>,
	offset: Option<u64>,
	keyword: Option<String>,

	#[serde(default)]
	friends_only: bool,
}

#[derive(AppRoute, Debug, PartialEq)]
#[route("/users")]
struct UsersListWithQuery {
	#[query]
	query: UserListQuery,
}

#[test]
fn no_params_simple_query_required() {
	let path: Result<UsersListWithQuery, _> = "/users".parse();
	match path {
		Err(RouteParseErr::NoQueryString) => {}
		_ => assert!(false),
	}
}

#[test]
fn no_params_simple_query() {
	let path: UsersListWithQuery = "/users?friends_only=true".parse().unwrap();
	assert_eq!(
		path,
		UsersListWithQuery {
			query: {
				UserListQuery {
					limit: None,
					offset: None,
					keyword: None,
					friends_only: true,
				}
			}
		}
	);
}

#[test]
fn no_params_simple_query_missing_bool_field() {
	let path: UsersListWithQuery = "/users?".parse().unwrap();
	assert_eq!(
		path,
		UsersListWithQuery {
			query: {
				UserListQuery {
					limit: None,
					offset: None,
					keyword: None,
					friends_only: false,
				}
			}
		}
	);
}

#[test]
fn no_params_simple_query_invalid_type() {
	let path: Result<UsersListWithQuery, _> = "/users?offset=test".parse();
	match path {
		Err(RouteParseErr::QueryParseErr(_)) => {}
		_ => assert!(false),
	}
}

#[test]
fn no_params_simple_query_all_defined() {
	let path: UsersListWithQuery =
		"/users?offset=10&limit=20&friends_only=false&keyword=some_keyword"
			.parse()
			.unwrap();
	assert_eq!(
		path,
		UsersListWithQuery {
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
}

#[test]
fn no_params_simple_query_url_decoding() {
	let path: UsersListWithQuery = "/users?keyword=some%20keyword%20with%20ampersand-question-equals-stuff%26%3F%3d%3a%3b%40%23%25%5e%5b%5d%7b%7D%60%22%3c%3e%E6%97%A5%E6%9C%AC%E8%AA%9E".parse().unwrap();
	assert_eq!(
		path,
		UsersListWithQuery {
			query: UserListQuery {
				limit: None,
				offset: None,
				keyword: Some(
					"some keyword with ampersand-question-equals-stuff&?=:;@#%^[]{}`\"<>日本語"
						.to_string()
				),
				friends_only: false,
			}
		}
	);
}

// TODO - uncomment after https://github.com/samscott89/serde_qs/pull/18
// #[test]
// fn no_params_simple_query_url_decoding_plus_sign() {
//     let path = UsersListWithQuery::from_str("/users?keyword=%2b").unwrap();
//     assert_eq!(path, UsersListWithQuery { query: {
//         UserListQuery {
//             limit: None,
//             offset: None,
//             keyword: Some("+".to_string()),
//             friends_only: false,
//         }
//     } });
// }

#[derive(AppRoute, Debug, PartialEq)]
#[route("/users/:user_id")]
struct UserDetailExtraPath {
	user_id: u8,

	#[query]
	query: Option<UserListQuery>,
}

#[test]
fn one_param_optional_query_missing() {
	let path: UserDetailExtraPath = "/users/8".parse().unwrap();
	assert_eq!(
		path,
		UserDetailExtraPath {
			user_id: 8,
			query: None
		}
	);
}

#[test]
fn one_param_optional_query_present() {
	let path: UserDetailExtraPath = "/users/8?limit=55".parse().unwrap();
	assert_eq!(
		path,
		UserDetailExtraPath {
			user_id: 8,
			query: Some(UserListQuery {
				limit: Some(55),
				offset: None,
				keyword: None,
				friends_only: false,
			})
		}
	);
}

#[test]
fn one_param_num_out_of_range() {
	let path: Result<UserDetailExtraPath, _> = "/users/256".parse();
	match path {
		Err(RouteParseErr::ParamParseErr(_)) => {}
		_ => assert!(false),
	}
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Building {
	name: String,
	number: Option<u32>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Country {
	CountryA,
	CountryB,
	CountryC,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Address {
	street_name: Option<String>,
	apt_number: Option<u32>,
	country: Option<Country>,
	building: Option<Building>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ParentQuery {
	// Not sure why you'd have an address here, but I had to think of
	// _something_ nested
	address: Option<Address>,
}

#[derive(AppRoute, Debug, PartialEq)]
#[route("/users/:user_id")]
struct UserDetailNestedQueryPath {
	user_id: u32,

	#[query]
	query: Option<ParentQuery>,
}

#[test]
fn nested_query_1() {
	let path: UserDetailNestedQueryPath = "/users/1024?address[apt_number]=101".parse().unwrap();
	assert_eq!(
		path,
		UserDetailNestedQueryPath {
			user_id: 1024,
			query: Some(ParentQuery {
				address: Some(Address {
					street_name: None,
					apt_number: Some(101),
					country: None,
					building: None,
				})
			})
		}
	);
}

#[test]
fn nested_query_2() {
	let path: UserDetailNestedQueryPath =
		"/users/1024?address[apt_number]=101&address[country]=country_b"
			.parse()
			.unwrap();
	assert_eq!(
		path,
		UserDetailNestedQueryPath {
			user_id: 1024,
			query: Some(ParentQuery {
				address: Some(Address {
					street_name: None,
					apt_number: Some(101),
					country: Some(Country::CountryB),
					building: None,
				})
			})
		}
	);
}

#[test]
fn nested_query_3() {
	let path: UserDetailNestedQueryPath = "/users/1024?address[apt_number]=101&address[country]=country_b&address[building][name]=Cool%20Building".parse().unwrap();
	assert_eq!(
		path,
		UserDetailNestedQueryPath {
			user_id: 1024,
			query: Some(ParentQuery {
				address: Some(Address {
					street_name: None,
					apt_number: Some(101),
					country: Some(Country::CountryB),
					building: Some(Building {
						name: "Cool Building".to_string(),
						number: None,
					})
				})
			})
		}
	);
}

#[test]
fn nested_query_4() {
	let path: UserDetailNestedQueryPath = "/users/1024?address[apt_number]=101&address[country]=country_b&address[building][name]=Cool%20Building&address[building][number]=not_number".parse().unwrap();
	assert_eq!(
		path,
		UserDetailNestedQueryPath {
			user_id: 1024,
			query: None,
		}
	);
}

#[test]
fn nested_query_5() {
	let path: UserDetailNestedQueryPath = "/users/1024?address[apt_number]=101&address[country]=country_b&address[building][name]=Cool%20Building&address[building][number]=9000".parse().unwrap();
	assert_eq!(
		path,
		UserDetailNestedQueryPath {
			user_id: 1024,
			query: Some(ParentQuery {
				address: Some(Address {
					street_name: None,
					apt_number: Some(101),
					country: Some(Country::CountryB),
					building: Some(Building {
						name: "Cool Building".to_string(),
						number: Some(9000),
					})
				})
			})
		}
	);
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct VecQuery {
	friend_ids: Vec<u32>,
}

#[derive(AppRoute, Debug, PartialEq)]
#[route("/users/:user_id")]
struct UserDetailVecQueryPath {
	user_id: u32,

	#[query]
	query: Option<VecQuery>,
}

#[test]
fn vec_query_1() {
	let path: UserDetailVecQueryPath = "/users/1024?".parse().unwrap();
	assert_eq!(
		path,
		UserDetailVecQueryPath {
			user_id: 1024,
			query: None,
		}
	);
}

#[test]
fn vec_query_2() {
	let path: UserDetailVecQueryPath = "/users/1024?friend_ids".parse().unwrap();
	assert_eq!(
		path,
		UserDetailVecQueryPath {
			user_id: 1024,
			query: None,
		}
	);
}

#[test]
fn vec_query_3() {
	let path: UserDetailVecQueryPath = "/users/1024?friend_ids[]=1".parse().unwrap();
	assert_eq!(
		path,
		UserDetailVecQueryPath {
			user_id: 1024,
			query: Some(VecQuery {
				friend_ids: vec![1],
			}),
		}
	);
}

#[test]
fn vec_query_4() {
	let path: UserDetailVecQueryPath = "/users/1024?friend_ids[]=1&friend_ids[]=20"
		.parse()
		.unwrap();
	assert_eq!(
		path,
		UserDetailVecQueryPath {
			user_id: 1024,
			query: Some(VecQuery {
				friend_ids: vec![1, 20],
			}),
		}
	);
}

#[test]
fn vec_query_5() {
	let path: UserDetailVecQueryPath = "/users/1024?friend_ids[]=1&friend_ids[]=20&friend_ids=33"
		.parse()
		.unwrap();
	assert_eq!(
		path,
		UserDetailVecQueryPath {
			user_id: 1024,
			query: None,
		}
	);
}

#[test]
fn vec_query_6() {
	let path: UserDetailVecQueryPath =
		"/users/1024?friend_ids[0]=1&friend_ids[1]=20&friend_ids[2]=33"
			.parse()
			.unwrap();
	assert_eq!(
		path,
		UserDetailVecQueryPath {
			user_id: 1024,
			query: Some(VecQuery {
				friend_ids: vec![1, 20, 33],
			}),
		}
	);
}

#[test]
fn vec_query_7() {
	let path: UserDetailVecQueryPath =
		"/users/1024?friend_ids[1]=20&friend_ids[2]=33&friend_ids[0]=1"
			.parse()
			.unwrap();
	assert_eq!(
		path,
		UserDetailVecQueryPath {
			user_id: 1024,
			query: Some(VecQuery {
				friend_ids: vec![1, 20, 33],
			}),
		}
	);
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
	Asc,
	Desc,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct SubmissionsQuery {
	column: Option<String>,
	direction: Option<SortDirection>,
	keyword: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct LimitOffsetQuery {
	limit: Option<u64>,
	offset: Option<u64>,
}

#[derive(AppRoute, Debug, PartialEq)]
#[route("/p/:project_id/exams/:exam_id/submissions_expired")]
struct ExpiredSubmissionsPath {
	project_id: String,
	exam_id: u64,

	#[query]
	query: std::option::Option<SubmissionsQuery>,

	#[query]
	limit: Option<LimitOffsetQuery>,
}

#[test]
fn test_no_query() {
	let path: ExpiredSubmissionsPath = "/p/43/exams/10/submissions_expired".parse().unwrap();
	assert_eq!(
		path,
		ExpiredSubmissionsPath {
			project_id: "43".to_string(),
			exam_id: 10,
			query: None,
			limit: None,
		}
	);
}

#[test]
fn test_only_question_mark() {
	let path: ExpiredSubmissionsPath = "/p/43/exams/10/submissions_expired?".parse().unwrap();
	assert_eq!(
		path,
		ExpiredSubmissionsPath {
			project_id: "43".to_string(),
			exam_id: 10,
			query: Some(SubmissionsQuery {
				column: None,
				direction: None,
				keyword: None,
			}),
			limit: Some(LimitOffsetQuery {
				limit: None,
				offset: None,
			}),
		}
	);
}
