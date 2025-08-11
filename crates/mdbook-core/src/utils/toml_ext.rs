//! Helper for working with toml types.

use toml::value::{Table, Value};

/// Helper for working with toml types.
pub(crate) trait TomlExt {
    /// Read a dotted key.
    fn read(&self, key: &str) -> Option<&Value>;
    /// Insert with a dotted key.
    fn insert(&mut self, key: &str, value: Value);
}

impl TomlExt for Value {
    fn read(&self, key: &str) -> Option<&Value> {
        if let Some((head, tail)) = split(key) {
            self.get(head)?.read(tail)
        } else {
            self.get(key)
        }
    }

    fn insert(&mut self, key: &str, value: Value) {
        if !self.is_table() {
            *self = Value::Table(Table::new());
        }

        let table = self.as_table_mut().expect("unreachable");

        if let Some((head, tail)) = split(key) {
            table
                .entry(head)
                .or_insert_with(|| Value::Table(Table::new()))
                .insert(tail, value);
        } else {
            table.insert(key.to_string(), value);
        }
    }
}

fn split(key: &str) -> Option<(&str, &str)> {
    let ix = key.find('.')?;

    let (head, tail) = key.split_at(ix);
    // splitting will leave the "."
    let tail = &tail[1..];

    Some((head, tail))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_simple_table() {
        let src = "[table]";
        let value: Value = toml::from_str(src).unwrap();

        let got = value.read("table").unwrap();

        assert!(got.is_table());
    }

    #[test]
    fn read_nested_item() {
        let src = "[table]\nnested=true";
        let value: Value = toml::from_str(src).unwrap();

        let got = value.read("table.nested").unwrap();

        assert_eq!(got, &Value::Boolean(true));
    }

    #[test]
    fn insert_item_at_top_level() {
        let mut value = Value::Table(Table::default());
        let item = Value::Boolean(true);

        value.insert("first", item.clone());

        assert_eq!(value.get("first").unwrap(), &item);
    }

    #[test]
    fn insert_nested_item() {
        let mut value = Value::Table(Table::default());
        let item = Value::Boolean(true);

        value.insert("first.second", item.clone());

        let inserted = value.read("first.second").unwrap();
        assert_eq!(inserted, &item);
    }
}
