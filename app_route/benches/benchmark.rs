#[macro_use]
extern crate criterion;

use app_route::AppRoute;
use criterion::Criterion;
use serde::{Deserialize, Serialize};

// Trivial case
#[derive(AppRoute, Debug, PartialEq)]
#[route("/users")]
struct UsersListPath {}

fn trivial_benchmark(c: &mut Criterion) {
	c.bench_function("UsersListPath", |b| {
		b.iter(|| {
			let _path: UsersListPath = "/users".parse().unwrap();
		})
	});
}

// Simple case
#[derive(AppRoute, Debug, PartialEq)]
#[route("/users/:user_id")]
struct UserDetailPath {
	user_id: u64,
}

fn simple_benchmark(c: &mut Criterion) {
	c.bench_function("UserDetailPath", |b| {
		b.iter(|| {
			let _path: UserDetailPath = "/users/642151".parse().unwrap();
		})
	});
}

// Nested case
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
	address: Option<Address>,
}

#[derive(AppRoute, Debug, PartialEq)]
#[route("/users/:user_id")]
struct UserDetailNestedQueryPath {
	user_id: u32,

	#[query]
	query: Option<ParentQuery>,
}

fn nested_benchmark(c: &mut Criterion) {
	c.bench_function("UserDetailNestedQueryPath", |b| b.iter(|| {
        let _path: UserDetailNestedQueryPath = "/users/1024?address[apt_number]=101&address[country]=country_b&address[building][name]=Cool%20Building&address[building][number]=9000".parse().unwrap();
    }));
}

// Vector case
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

fn vec_benchmark(c: &mut Criterion) {
	c.bench_function("UserDetailVecQueryPath", |b| {
		b.iter(|| {
			let _path: UserDetailVecQueryPath =
				"/users/1024?friend_ids[1]=20&friend_ids[2]=33&friend_ids[0]=1"
					.parse()
					.unwrap();
		})
	});
}

criterion_group!(
	benches,
	trivial_benchmark,
	simple_benchmark,
	nested_benchmark,
	vec_benchmark
);
criterion_main!(benches);
