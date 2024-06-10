# toml_pretty

A function to pretty serialize a serde-serializable value to toml.

Can serialize structs to toml in a single block (unlike the `toml` crate, which is great for deserialization but not so great for pretty serialization)

Nested array fields more than 2 arrays deep are not supported.

Note. All items in arrays are on a new line and indented. `toml_pretty::to_string` uses `\t` as tab.
An alternal tab symbol can be used (eg. 2 spaces) using `toml_pretty::to_string_custom_tab`.

## Example

Given serializable structs:
```
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
```

Can use `toml_pretty::to_string`:

```
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
	toml_pretty::to_string(&user)
		.context("failed to ser")
		.unwrap()
);
```