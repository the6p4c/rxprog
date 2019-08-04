use std::collections::HashMap;
use std::convert::TryFrom;

const KV_PAIR_DELIMETER: char = ';';
const KV_DELIMETER: char = '=';

#[derive(Debug, PartialEq)]
pub struct ConnectionString<'a> {
    data: HashMap<&'a str, &'a str>,
}

impl<'a> TryFrom<&'a str> for ConnectionString<'a> {
    type Error = &'static str;

    fn try_from(s: &'a str) -> Result<ConnectionString<'a>, &'static str> {
        let pairs = s
            .split(KV_PAIR_DELIMETER)
            .map(|kv_pair| {
                // No point unnecessirally rejecting a connection string that
                // looks like "a=b;;c=d", so skip over a key/value pair if it's
                // empty
                if kv_pair.len() == 0 {
                    return Ok(None);
                }

                let mut kv_parts = kv_pair.split(KV_DELIMETER);
                match (kv_parts.next(), kv_parts.next(), kv_parts.next()) {
                    // Don't accept a key/value pair without an =
                    (Some(_), None, _) => Err("no key/value delimeter"),
                    // Ensure there's only two elements, i.e accept "x=y" but
                    // not "x=y=z"
                    (Some(key), Some(value), None) => {
                        if key.len() == 0 {
                            Err("empty key")
                        } else {
                            Ok(Some((key, value)))
                        }
                    }
                    _ => Err("more than one key/value delimeter in one key/value pair"),
                }
            })
            // Take first error (Result::transpose) and eliminate Ok(None)
            // values (filter_map)
            .filter_map(Result::transpose)
            .collect::<Result<Vec<_>, _>>()?;

        // Check for duplicate keys
        let mut data = HashMap::new();
        for (key, value) in pairs.iter() {
            if data.contains_key(key) {
                return Err("duplicate key");
            }

            data.insert(*key, *value);
        }

        Ok(ConnectionString { data })
    }
}

impl ConnectionString<'_> {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|value| *value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        let cs = ConnectionString::try_from("");

        assert_eq!(
            cs,
            Ok(ConnectionString {
                data: HashMap::new()
            })
        );
    }

    #[test]
    fn one_kv_pair() {
        let cs = ConnectionString::try_from("a=b").unwrap();

        assert_eq!(*cs.data.get("a").unwrap(), "b");
    }

    #[test]
    fn two_kv_pairs() {
        let cs = ConnectionString::try_from("a=b;c=d").unwrap();

        assert_eq!(*cs.data.get("a").unwrap(), "b");
        assert_eq!(*cs.data.get("c").unwrap(), "d");
    }

    #[test]
    fn empty_value() {
        let cs = ConnectionString::try_from("a=;c=d").unwrap();

        assert_eq!(*cs.data.get("a").unwrap(), "");
        assert_eq!(*cs.data.get("c").unwrap(), "d");
    }

    #[test]
    fn empty_key() {
        let cs = ConnectionString::try_from("=b;c=d");

        assert_eq!(cs, Err("empty key"));
    }

    #[test]
    fn duplicate_key() {
        let cs = ConnectionString::try_from("a=b;c=d;a=f");

        assert_eq!(cs, Err("duplicate key"));
    }

    #[test]
    fn no_kv_delimeters() {
        let cs = ConnectionString::try_from("a;c=d");

        assert_eq!(cs, Err("no key/value delimeter"));
    }

    #[test]
    fn too_many_kv_delimeters() {
        let cs = ConnectionString::try_from("a=b=c;c=d");

        assert_eq!(
            cs,
            Err("more than one key/value delimeter in one key/value pair")
        );
    }

    #[test]
    fn get_unknown_key() {
        let cs = ConnectionString::try_from("a=b;c=d").unwrap();

        let value = cs.get("e");

        assert_eq!(value, None);
    }

    #[test]
    fn get_key() {
        let cs = ConnectionString::try_from("a=b;c=d").unwrap();

        let value = cs.get("a");

        assert_eq!(value, Some("b"));
    }
}
