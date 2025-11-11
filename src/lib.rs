use std::fmt::Write;

use ordered_hash_map::OrderedHashMap;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Failed to de/serialize value to json")]
  JsonSerialization(#[from] serde_json::Error),
  #[error("Failed to format args")]
  Format(#[from] std::fmt::Error),
  #[error("Came across triple nested array. Not supported.")]
  TripleNestedArray,
  #[error("Came across Value::Object after flatten_map. This shouldn't happen")]
  ObjectReached,
}

#[derive(Clone, Copy)]
pub struct Options<'a> {
  pub tab: &'a str,
  pub skip_empty_string: bool,
  pub skip_empty_object: bool,
  pub inline_array: bool,
  pub max_inline_array_length: usize,
}

impl<'a> Default for Options<'a> {
  fn default() -> Self {
    Self {
      tab: "\t",
      skip_empty_string: false,
      skip_empty_object: false,
      inline_array: false,
      max_inline_array_length: 50,
    }
  }
}

impl<'a> Options<'a> {
  /// Specify the symbol to use for tab. Default is '\t'
  pub fn tab(mut self, tab: &'a str) -> Self {
    self.tab = tab;
    self
  }

  /// Specify whether to skip serializing string fields containing empty strings
  pub fn skip_empty_string(mut self, skip_empty_string: bool) -> Self {
    self.skip_empty_string = skip_empty_string;
    self
  }

  /// Specify whether to skip serializing object fields containing empty objects
  pub fn skip_empty_object(mut self, skip_empty_object: bool) -> Self {
    self.skip_empty_object = skip_empty_object;
    self
  }

  /// Specify whether to serialize arrays inline, rather than on multiple lines.
  pub fn inline_array(mut self, inline_array: bool) -> Self {
    self.inline_array = inline_array;
    self
  }

  pub fn max_inline_array_length(mut self, max_inline_array_length: usize) -> Self {
    self.max_inline_array_length = max_inline_array_length;
    self
  }
}

pub fn to_string<T: Serialize>(value: &T, options: Options<'_>) -> Result<String> {
  let Options {
    tab,
    skip_empty_string,
    skip_empty_object,
    inline_array,
    max_inline_array_length,
  } = options;
  let map = serde_json::from_str(&serde_json::to_string(value).map_err(Error::JsonSerialization)?)
    .map_err(Error::JsonSerialization)?;
  let mut res = String::new();
  for (i, (key, val)) in flatten_map(map, skip_empty_object).into_iter().enumerate() {
    match &val {
      Value::Null => {}

      Value::Bool(_) | Value::Number(_) => {
        if i != 0 {
          res.push('\n');
        }
        res
          .write_fmt(format_args!("{key} = {val}"))
          .map_err(Error::Format)?;
      }

      Value::String(val) => {
        if skip_empty_string && val.is_empty() {
          continue;
        }
        if i != 0 {
          res.push('\n');
        }
        if val.contains('\n') {
          res
            .write_fmt(format_args!("{key} = \"\"\"\n{val}\"\"\""))
            .map_err(Error::Format)?;
        } else {
          res
            .write_fmt(format_args!("{key} = \"{}\"", val.replace('"', "\\\"")))
            .map_err(Error::Format)?;
        }
      }

      Value::Array(vals) => {
        if vals.is_empty() {
          if i != 0 {
            res.push('\n');
          }
          res
            .write_fmt(format_args!("{key} = []"))
            .map_err(Error::Format)?;
          continue;
        }
        let mut strs = Vec::<String>::with_capacity(vals.capacity());
        for val in vals {
          match val {
            Value::Null => {}
            Value::Bool(_) | Value::Number(_) => strs.push(val.to_string()),
            Value::String(string) => {
              if skip_empty_string && string.is_empty() {
                continue;
              }
              strs.push(format!("\"{}\"", string.replace('"', "\\\"")))
            }
            Value::Object(map) => strs.push(to_array_object_string(&map, options)?),
            Value::Array(vals) => {
              let mut out = Vec::new();
              for val in vals {
                match val {
                  Value::Null => {}
                  Value::Bool(_) | Value::Number(_) => out.push(val.to_string()),
                  Value::String(string) => out.push(format!("\"{}\"", string.replace('"', "\\\""))),
                  Value::Object(map) => out.push(to_array_object_string(&map, options)?),
                  Value::Array(_) => return Err(Error::TripleNestedArray),
                }
              }
              strs.push(format!("[{}]", out.join(", ")));
            }
          }
        }
        let total_length = strs.iter().fold(0, |total, curr| total + curr.len());
        let inline_array = inline_array || total_length <= max_inline_array_length;
        let join = if inline_array {
          String::from(", ")
        } else {
          format!(",\n{tab}")
        };
        let val = strs.join(&join);
        if i != 0 {
          res.push('\n');
        }
        if inline_array {
          res
            .write_fmt(format_args!("{key} = [{val}]"))
            .map_err(Error::Format)?;
        } else {
          res
            .write_fmt(format_args!("{key} = [\n{tab}{val}\n]"))
            .map_err(Error::Format)?;
        }
      }

      // Special Object case for including empty objects
      Value::Object(obj) if !skip_empty_object && obj.is_empty() => {
        if i != 0 {
          res.push('\n');
        }
        // Write empty object eg 'database = {}'
        res
          .write_fmt(format_args!("{key} = {{}}"))
          .map_err(Error::Format)?;
      }

      // All other object cases should be removed by flatten_map
      Value::Object(_) => return Err(Error::ObjectReached),
    }
  }
  Ok(res)
}

fn flatten_map(
  map: OrderedHashMap<String, Value>,
  skip_empty_object: bool,
) -> OrderedHashMap<String, Value> {
  let mut target = OrderedHashMap::new();
  flatten_map_rec(&mut target, None, map, skip_empty_object);
  target
}

fn flatten_map_rec(
  target: &mut OrderedHashMap<String, Value>,
  parent_field: Option<String>,
  source: OrderedHashMap<String, Value>,
  skip_empty_object: bool,
) {
  if !skip_empty_object && source.is_empty() {
    if let Some(parent_field) = &parent_field {
      target.insert(
        parent_field.to_string(),
        Value::Object(serde_json::Map::new()),
      );
      return;
    }
  }
  for (field, val) in source {
    let parent_field = if let Some(parent_field) = &parent_field {
      let mut parent_field = parent_field.clone();
      parent_field.push('.');
      parent_field.push_str(&field);
      parent_field
    } else {
      field
    };
    if let Value::Object(source) = val {
      flatten_map_rec(
        target,
        Some(parent_field),
        source.into_iter().collect(),
        skip_empty_object,
      )
    } else {
      target.insert(parent_field, val);
    }
  }
}

/// Serializes to:
///
/// ```toml
/// { name = "asdf", pattern = """
/// asdf
/// bbbb""", last = true }
/// ```
fn to_array_object_string(
  map: &serde_json::Map<String, Value>,
  options: Options<'_>,
) -> Result<String> {
  let mut res = String::new();
  for token in to_string(&map, options.inline_array(true))?
    .split(" = ")
    .map(str::trim)
  {
    // Split into val / next key, if exists
    match token.rsplit_once('\n') {
      Some((val, next_key)) => {
        res.push_str(" = ");
        res.push_str(val.trim());
        res.push_str(", ");
        res.push_str(next_key.trim());
      }
      None => {
        if res.is_empty() {
          // first key
          res.push_str(token);
        } else {
          // last val
          res.push_str(" = ");
          res.push_str(token);
        }
      }
    }
  }
  Ok(format!("{{ {res} }}"))
}
