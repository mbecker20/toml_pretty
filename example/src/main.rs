use std::collections::HashMap;

use anyhow::Context;
use serde::Serialize;
use toml_pretty::Options;

#[derive(Serialize)]
struct User {
  name: String,
  nicknames: Vec<String>,
  birthday: Birthday,
  more: Vec<Birthday>,
  empty: HashMap<String, String>,
}

#[derive(Serialize)]
struct Birthday {
  day: u8,
  month: u8,
  year: u16,
}

fn main() {
  let user = User {
    name: String::from("Jonathan"),
    nicknames: vec![String::from("Jack"), String::from("Jon")],
    birthday: Birthday {
      day: 0,
      month: 0,
      year: 1980,
    },
    more: vec![
      Birthday {
        day: 0,
        month: 0,
        year: 1980,
      },
      Birthday {
        day: 0,
        month: 0,
        year: 1980,
      },
    ],
    empty: HashMap::new(),
  };
  println!(
    "{}",
    toml_pretty::to_string(
      &user,
      Options::default()
        .tab("  ")
        .skip_empty_string(true)
        .skip_empty_object(false)
    )
    .context("failed to ser")
    .unwrap()
  );
}
