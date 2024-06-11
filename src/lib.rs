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

pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
  to_string_custom_tab(value, "\t")
}

/// This function only supports serializable structs, as top level toml
/// requires a map.
pub fn to_string_custom_tab<T: Serialize>(value: &T, tab: &str) -> Result<String> {
  let map = serde_json::from_str(&serde_json::to_string(value).map_err(Error::JsonSerialization)?)
    .map_err(Error::JsonSerialization)?;
  let mut res = String::new();
  for (i, (key, val)) in flatten_map(map).into_iter().enumerate() {
    if i != 0 {
      res.push('\n');
    }
    match &val {
      Value::Null => {}

      Value::Bool(_) | Value::Number(_) => {
        res
          .write_fmt(format_args!("{key} = {val}"))
          .map_err(Error::Format)?;
      }

      Value::String(val) => {
        if val.contains('\n') {
          res
            .write_fmt(format_args!("{key} = \"\"\"\n{val}\n\"\"\""))
            .map_err(Error::Format)?;
        } else {
          res
            .write_fmt(format_args!("{key} = \"{val}\""))
            .map_err(Error::Format)?;
        }
      }

      Value::Array(vals) => {
        if vals.is_empty() {
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
            Value::String(string) => strs.push(string.to_owned()),
            Value::Object(map) => strs.push(format!(
              "{{ {} }}",
              to_string(&map)?.split('\n').collect::<Vec<_>>().join(", ")
            )),
            Value::Array(vals) => {
              let mut out = Vec::new();
              for val in vals {
                match val {
                  Value::Null => {}
                  Value::Bool(_) | Value::Number(_) | Value::String(_) => out.push(val.to_string()),
                  Value::Object(map) => out.push(format!(
                    "{{ {} }}",
                    to_string(&map)?.split('\n').collect::<Vec<_>>().join(", ")
                  )),
                  Value::Array(_) => return Err(Error::TripleNestedArray),
                }
              }
              strs.push(format!("[{}]", out.join(", ")));
            }
          }
        }
        let val = strs.join(&format!(",\n{tab}"));
        res
          .write_fmt(format_args!("{key} = [\n{tab}{val}\n]"))
          .map_err(Error::Format)?;
      }

      // All objects should be removed by flatten_map
      Value::Object(_) => return Err(Error::ObjectReached),
    }
  }
  Ok(res)
}

fn flatten_map(map: OrderedHashMap<String, Value>) -> OrderedHashMap<String, Value> {
  let mut target = OrderedHashMap::new();
  flatten_map_rec(&mut target, None, map);
  target
}

fn flatten_map_rec(
  target: &mut OrderedHashMap<String, Value>,
  parent_field: Option<String>,
  source: OrderedHashMap<String, Value>,
) {
  let parent_field = match parent_field {
    Some(mut parent_field) => {
      parent_field.push('.');
      parent_field
    }
    None => String::new(),
  };
  for (field, val) in source {
    let parent_field = if parent_field.is_empty() {
      field
    } else {
      let mut parent_field = parent_field.clone();
      parent_field.push_str(&field);
      parent_field
    };
    if let Value::Object(source) = val {
      flatten_map_rec(target, Some(parent_field), source.into_iter().collect())
    } else {
      target.insert(parent_field, val);
    }
  }
}

// Flattens a nested bson document using the mongo '.' syntax. Useful for partial updates.
// doc! { "f1": "yes", "f2": { "f3": "no" } } -> doc! { "f1": "yes", "f2.f3": "no" }
// pub fn flatten_document(doc: Document) -> Document {
//   let mut target = Document::new();
//   flatten_document_rec(&mut target, None, doc);
//   target
// }
