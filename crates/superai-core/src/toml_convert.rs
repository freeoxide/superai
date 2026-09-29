//! Shared TOML/JSON conversion: one walker set for the MCP destination
//! files (EXT-08..10) and the provider renderer (PRV), so a conversion
//! fix lands once.

use serde_json::{Map, Value};

/// Parse a TOML document into its JSON-equivalent value. `Item::None`
/// entries (phantom keys left by removals) are skipped; non-finite
/// floats become `Value::Null`, the same rendering `serde_json` uses.
pub(crate) fn document_to_value(doc: &toml_edit::DocumentMut) -> Value {
    table_to_value(doc.as_table())
}

fn table_to_value(table: &toml_edit::Table) -> Value {
    let mut map = Map::new();
    for (key, item) in table {
        if item.is_none() {
            continue;
        }
        map.insert(key.to_owned(), item_to_value(item));
    }
    Value::Object(map)
}

fn item_to_value(item: &toml_edit::Item) -> Value {
    match item {
        toml_edit::Item::Value(v) => toml_value_to_value(v),
        toml_edit::Item::Table(t) => table_to_value(t),
        toml_edit::Item::ArrayOfTables(a) => Value::Array(a.iter().map(table_to_value).collect()),
        toml_edit::Item::None => Value::Null,
    }
}

fn toml_value_to_value(v: &toml_edit::Value) -> Value {
    use toml_edit::Value as Tv;
    match v {
        Tv::String(s) => Value::String(s.value().to_owned()),
        Tv::Integer(i) => Value::Number((*i.value()).into()),
        Tv::Float(f) => Value::from(*f.value()),
        Tv::Boolean(b) => Value::Bool(*b.value()),
        Tv::Datetime(d) => Value::String(d.to_string()),
        Tv::Array(a) => Value::Array(a.iter().map(toml_value_to_value).collect()),
        Tv::InlineTable(t) => {
            let mut map = Map::new();
            for (key, value) in t {
                map.insert(key.to_owned(), toml_value_to_value(value));
            }
            Value::Object(map)
        }
    }
}

fn number_to_toml_value(n: &serde_json::Number) -> toml_edit::Value {
    if let Some(i) = n.as_i64() {
        toml_edit::Value::from(i)
    } else {
        toml_edit::Value::from(n.as_f64().unwrap_or_default())
    }
}

/// Server-entry flavor: null is refused, objects become inline tables,
/// and array members must themselves be values (error text is part of
/// the MCP diagnostics contract).
pub(crate) fn json_value_to_toml_strict(value: &Value) -> Result<toml_edit::Item, String> {
    match value {
        Value::Null => Err("toml cannot represent null in mcp server entries".to_owned()),
        Value::Bool(b) => Ok(toml_edit::Item::Value(toml_edit::Value::from(*b))),
        Value::Number(n) => Ok(toml_edit::Item::Value(number_to_toml_value(n))),
        Value::String(s) => Ok(toml_edit::Item::Value(toml_edit::Value::from(s.as_str()))),
        Value::Array(arr) => {
            let mut toml_arr = toml_edit::Array::new();
            for item in arr {
                let toml_edit::Item::Value(converted) = json_value_to_toml_strict(item)? else {
                    return Err("nested tables inside mcp arrays are not supported".to_owned());
                };
                toml_arr.push(converted);
            }
            Ok(toml_edit::Item::Value(toml_edit::Value::from(toml_arr)))
        }
        Value::Object(map) => {
            let mut table = toml_edit::Table::new();
            for (k, v) in map {
                table.insert(k.as_str(), json_value_to_toml_strict(v)?);
            }
            Ok(toml_edit::Item::Value(toml_edit::Value::InlineTable(
                table.into_inline_table(),
            )))
        }
    }
}

/// Engine-operation flavor: null renders as `Item::None`, objects as
/// sub-tables, and non-scalar array members as empty strings.
pub(crate) fn json_value_to_toml_lenient(value: &Value) -> toml_edit::Item {
    match value {
        Value::Null => toml_edit::Item::None,
        Value::Bool(b) => toml_edit::Item::Value(toml_edit::Value::from(*b)),
        Value::Number(n) => toml_edit::Item::Value(number_to_toml_value(n)),
        Value::String(s) => toml_edit::Item::Value(toml_edit::Value::from(s.as_str())),
        Value::Array(arr) => {
            let items: Vec<toml_edit::Value> = arr
                .iter()
                .map(|v| match json_value_to_toml_lenient(v) {
                    toml_edit::Item::Value(value) => value,
                    _ => toml_edit::Value::from(""),
                })
                .collect();
            toml_edit::Item::Value(toml_edit::Value::Array(items.into_iter().collect()))
        }
        Value::Object(map) => {
            let mut table = toml_edit::Table::new();
            for (key, child) in map {
                table.insert(key.as_str(), json_value_to_toml_lenient(child));
            }
            toml_edit::Item::Table(table)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn document_round_trips_scalars_tables_and_array_of_tables() {
        let doc = "[owner]\nname = \"a\"\nport = 1\npi = 3.5\ndead = nan\n[[items]]\nid = \"x\"\n[[items]]\nid = \"y\"\n[owner.sub]\nflag = true\n"
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        let v = document_to_value(&doc);
        assert_eq!(v["owner"]["name"], json!("a"));
        assert_eq!(v["owner"]["port"], json!(1));
        assert_eq!(v["owner"]["pi"], json!(3.5));
        // Non-finite floats have no JSON number; both former walkers
        // rendered them as null.
        assert_eq!(v["owner"]["dead"], Value::Null);
        assert_eq!(v["items"][0]["id"], json!("x"));
        assert_eq!(v["items"][1]["id"], json!("y"));
        assert_eq!(v["owner"]["sub"]["flag"], json!(true));
    }

    #[test]
    fn strict_flavor_refuses_null_and_inlines_objects() {
        let null_err = json_value_to_toml_strict(&Value::Null).unwrap_err();
        assert!(null_err.contains("null"), "{null_err}");
        // Objects become inline tables so an array of servers stays flat.
        let item = json_value_to_toml_strict(&json!({"a": 1})).unwrap();
        assert!(matches!(
            item,
            toml_edit::Item::Value(toml_edit::Value::InlineTable(_))
        ));
        let item = json_value_to_toml_strict(&json!([{"a": 1}, "x"])).unwrap();
        assert!(matches!(
            item,
            toml_edit::Item::Value(toml_edit::Value::Array(_))
        ));
    }

    #[test]
    fn lenient_flavor_renders_null_none_tables_and_empty_strings() {
        assert!(matches!(
            json_value_to_toml_lenient(&Value::Null),
            toml_edit::Item::None
        ));
        assert!(matches!(
            json_value_to_toml_lenient(&json!({"a": 1})),
            toml_edit::Item::Table(_)
        ));
        let item = json_value_to_toml_lenient(&json!([{"a": 1}, null, 2]));
        let toml_edit::Item::Value(toml_edit::Value::Array(arr)) = item else {
            panic!("expected an array item");
        };
        let rendered: Vec<String> = arr.iter().map(ToString::to_string).collect();
        assert_eq!(rendered, vec!["\"\"", "\"\"", "2"]);
    }
}
