use anyhow::Context;
use serde::Serialize;

#[derive(Serialize)]
struct User {
  name: String,
  nicknames: Vec<String>,
  birthday: Birthday,
  more: Vec<Birthday>,
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
  };
  println!(
    "{}",
    toml_pretty::to_string_custom_tab(&user, "  ")
      .context("failed to ser")
      .unwrap()
  );
}