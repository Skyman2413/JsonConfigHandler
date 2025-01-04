use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Eq, PartialEq)]
enum JsonRoot {
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Json {
    data: JsonRoot,
}
impl Json {
    pub fn get(&self, key: &str) -> Result<JsonValue, JsonError> {
        let mut current_object = match &self.data {
            JsonRoot::Object(obj) => obj,
            _ => return Err(JsonError::ParseError("Can't go into not Object".to_string())),
        };
        let mut result: JsonValue = JsonValue::Object(current_object.clone());
        let mut can_take_next: bool = true;
        for part in key.split('.'){
            if !can_take_next {
                return Err(JsonError::ParseError(format!("There is no key {part}")))
            }
            current_object = match current_object.get(part) {
                Some(JsonValue::Object(obj)) => obj,
                Some(value) => {
                    result = value.clone();
                    can_take_next = false;
                    continue
                },
                None => return Err(JsonError::ParseError(format!("Can't go into not object at {part}"))),
            }
        }
        Ok(result)
    }

    pub fn set(&mut self, key: &str, value: JsonValue) -> Result<(), JsonError> {
        if key.trim().is_empty(){
            return Err(JsonError::ParseError("Can't set empty key".to_string()));
        }
        let parts: Vec<&str> = key.trim().split('.').collect();

        let mut current = match &mut self.data {
            JsonRoot::Object(obj) => obj,
            _ => return Err(JsonError::ParseError("Root is not an object.".to_string())),
        };

        for (i, part) in parts.iter().enumerate() {
            let part = *part;
            
            if !current.contains_key(part) {
                current.insert(part.to_string(), JsonValue::Object(HashMap::new()));
            }
            
            if i == parts.len() - 1 {
                current.insert(part.to_string(), value);
                return Ok(());
            }

            current = match current.get_mut(part) {
                Some(JsonValue::Object(ref mut obj)) => obj,
                _ => {
                    return Err(JsonError::ParseError(format!(
                        "Key `{}` is not an object.",
                        part
                    )));
                }
            };
        }

        Ok(())
    }
}

impl From<String> for Json {
    fn from(data: String) -> Json {
        let parsed = JsonValue::parse(&data).unwrap();
        match parsed {
            JsonValue::Object(map) => Json {
                data: JsonRoot::Object(map),
            },
            JsonValue::Array(arr) => Json {
                data: JsonRoot::Array(arr),
            },
            _ => panic!("Invalid JSON object: It should be an object or an array"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum JsonValue {
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
    String(String),
    Number(i64),
    Bool(bool),
    Null,
}
impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Object(obj) => {
                let mut elements: Vec<String> = vec![];
                for (key, value) in obj {
                    elements.push(format!("\"{}\": {}", key, value));
                }
                write!(f, "{{{}}}", elements.join(", "))
            }
            JsonValue::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", elements.join(", "))
            }
            JsonValue::String(s) => write!(f, "\"{}\"", s),
            JsonValue::Number(n) => write!(f, "{}", n),
            JsonValue::Bool(b) => write!(f, "{}", b),
            JsonValue::Null => write!(f, "null"),
        }
    }
}
impl JsonValue {
    pub fn parse(input: &str) -> Result<JsonValue, JsonError> {
        let input = input.trim();
        if input.starts_with('{') {
            Self::parse_object(input)
        } else if input.starts_with('[') {
            Self::parse_array(input)
        } else if input.starts_with('"') {
            Self::parse_string(input)
        } else if input.to_lowercase() == "true" || input.to_lowercase() == "false" {
            Self::parse_bool(input)
        } else if input == "null" {
            Ok(JsonValue::Null)
        } else {
            Self::parse_number(input)
        }
    }

    fn parse_object(input: &str) -> Result<JsonValue, JsonError> {
        let input = input.trim();
        if !input.starts_with('{') || !input.ends_with('}') {
            return Err(JsonError::ParseError("Invalid JSON object: must start with '{' and end with '}'".to_string()));
        }

        let mut elements: HashMap<String, JsonValue> = HashMap::new();
        let mut current_key = String::new();
        let mut current_value = String::new();
        let mut stack = Vec::new();
        let mut inside_string = false;
        let mut is_key = true;

        for ch in input[1..input.len() - 1].chars() {
            match ch {
                '"' if stack.is_empty() => {
                    inside_string = !inside_string;
                    if !is_key {
                        current_value.push(ch);
                    }
                }
                ':' if stack.is_empty() => {
                    is_key = false;
                }
                '[' | '{' if !inside_string => {
                    stack.push(ch);
                    if is_key {
                        current_key.push(ch)
                    } else {
                        current_value.push(ch)
                    }
                }
                ']' | '}' if !inside_string => {
                    if stack.pop() != Some(if ch == ']' { '[' } else { '{' }) {
                        return Err(JsonError::ParseError(format!("Invalid JSON object: {}", ch)));
                    }
                    if is_key {
                        current_key.push(ch);
                    } else {
                        current_value.push(ch);
                    }
                }
                ',' if stack.is_empty() && !inside_string => {
                    if elements
                        .insert(current_key.clone(), Self::parse(&current_value)?)
                        .is_some()
                    {
                        return Err(JsonError::ParseError("Invalid JSON object".to_string()));
                    };
                    is_key = true;
                    current_key.clear();
                    current_value.clear();
                }
                _ => {
                    if inside_string && is_key {
                        current_key.push(ch)
                    } else if !is_key {
                        current_value.push(ch);
                    }
                }
            }
        }
        if !current_key.is_empty() && !current_value.is_empty() {
            if elements
                .insert(current_key.clone(), Self::parse(&current_value)?)
                .is_some()
            {
                return Err(JsonError::ParseError("Invalid JSON object".to_string()));
            };
            current_key.clear();
            current_value.clear();
            is_key = true
        }
        if !is_key
            || !stack.is_empty()
            || !current_value.is_empty()
            || !current_key.is_empty()
            || inside_string
        {
            return Err(JsonError::ParseError(format!("Invalid JSON object: {}", input)));
        }

        Ok(JsonValue::Object(elements))
    }

    fn parse_array(input: &str) -> Result<JsonValue, JsonError> {
        let input = input.trim();
        if !input.starts_with('[') || !input.ends_with(']') {
            return Err(JsonError::ParseError("Invalid JSON array: must start with '[' and end with ']'".to_string()));
        }

        let mut elements = Vec::new();
        let mut current = String::new();
        let mut stack = Vec::new();
        let mut inside_string = false;

        for ch in input[1..input.len() - 1].chars() {
            match ch {
                '"' if stack.is_empty() => {
                    inside_string = !inside_string;
                    current.push(ch);
                }
                '[' | '{' if !inside_string => {
                    stack.push(ch);
                    current.push(ch);
                }
                ']' | '}' if !inside_string => {
                    if stack.pop() != Some(if ch == ']' { '[' } else { '{' }) {
                        return Err(JsonError::ParseError(format!("Invalid JSON array: {}", ch)));
                    }
                    current.push(ch);
                }
                ',' if stack.is_empty() && !inside_string => {
                    elements.push(Self::parse(&current.trim())?);
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }
        if !current.is_empty() {
            elements.push(Self::parse(&current)?);
            current.clear();
        }
        if inside_string {
            Err(JsonError::ParseError(format!("Invalid JSON array: {}", current)))
        } else if !stack.is_empty() {
            Err(JsonError::ParseError(format!("Invalid JSON array: {:?}", stack)))
        } else {
            Ok(JsonValue::Array(elements))
        }
    }

    fn parse_string(input: &str) -> Result<JsonValue, JsonError> {
        Ok(JsonValue::String(input.trim_matches('"').to_string()))
    }

    fn parse_number(input: &str) -> Result<JsonValue, JsonError> {
        Ok(input.parse::<i64>().map(JsonValue::Number).unwrap())
    }

    fn parse_bool(input: &str) -> Result<JsonValue, JsonError> {
        Ok(JsonValue::Bool(input == "true"))
    }
}

#[derive(Debug)]
pub enum JsonError{
    KeyNotFound(String),
    InvalidType(String),
    ParseError(String),
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_value() {
        let json_str = r#"{ "key1": { "key2": { "key3": "value" } } }"#;
        let json = Json::from(json_str.to_string());
        assert_eq!(
            json.get("key1.key2.key3").unwrap(),
            JsonValue::String("value".to_string())
        );
    }

    #[test]
    #[should_panic]
    fn test_incorrect_structure() {
        let input = "{\"Hello\": {[\"asdf\", 2134}]}";
        let json = JsonValue::parse(input).unwrap();
    }
    #[test]
    #[should_panic]
    fn test_duplicate_key() {
        let input = "{\"foo\": 1234, \"foo\": true}";
        let json = Json::from(input.to_string());
    }
    #[test]
    fn test_leveled_structure() {
        let input = "{\"struct\": {\"hello\": [\"world\", 24]}, \"my_life\": \"be live\"}";
        let json = Json::from(input.to_string());
        let mut expected_object = HashMap::new();
        let mut inside_obj = HashMap::new();
        inside_obj.insert(
            "hello".to_string(),
            JsonValue::Array(vec![
                JsonValue::String("world".parse().unwrap()),
                JsonValue::Number(24),
            ]),
        );
        expected_object.insert("struct".to_string(), JsonValue::Object(inside_obj));
        expected_object.insert(
            "my_life".to_string(),
            JsonValue::String("be live".to_string()),
        );
        let expected = Json {
            data: JsonRoot::Object(expected_object),
        };
        assert_eq!(json, expected);
    }
    #[test]
    fn test_all_simple_types() {
        let input = "{\"foo\": \"bar\", \"boo\": true, \"far\": 34, \"nil\": null}";
        let json = Json::from(input.to_string());
        let mut expected_object = HashMap::new();
        expected_object.insert("foo".to_string(), JsonValue::String("bar".to_string()));
        expected_object.insert("boo".to_string(), JsonValue::Bool(true));
        expected_object.insert("far".to_string(), JsonValue::Number(34));
        expected_object.insert("nil".to_string(), JsonValue::Null);
        let expected = Json {
            data: JsonRoot::Object(expected_object),
        };
        assert_eq!(json, expected);
    }
    #[test]
    fn one_array_test_with_all_simple_types() {
        let input = "{\"foo\": [\"asdf\", 1, null, true]}";
        let json = Json::from(input.to_string());
        let mut expected_object = HashMap::new();
        expected_object.insert(
            "foo".to_string(),
            JsonValue::Array(vec![
                JsonValue::String("asdf".to_string()),
                JsonValue::Number(1),
                JsonValue::Null,
                JsonValue::Bool(true),
            ]),
        );
        let expected = Json {
            data: JsonRoot::Object(expected_object),
        };
        assert_eq!(json, expected);
    }

    #[test]
    fn test_set_value() {
        let mut json = Json::from("{ \"key1\": { \"key2\": { \"key3\": \"value\" } } }".to_string());

        json.set("key1.key2.key4", JsonValue::Number(42)).unwrap();

        assert_eq!(
            json.get("key1.key2.key4").unwrap(),
            JsonValue::Number(42)
        );

        json.set("key1.key5", JsonValue::Bool(true)).unwrap();

        assert_eq!(
            json.get("key1.key5").unwrap(),
            JsonValue::Bool(true)
        );
    }

    #[test]
    fn test_set_simple_key() {
        let mut json = Json::from("{ \"key1\": \"value1\" }".to_string());

        json.set("key1", JsonValue::Number(42)).unwrap();

        assert_eq!(json.get("key1").unwrap(), JsonValue::Number(42));
    }

    #[test]
    fn test_set_new_key_in_existing_object() {
        let mut json = Json::from("{ \"key1\": { \"key2\": \"value2\" } }".to_string());

        json.set("key1.key3", JsonValue::Bool(true)).unwrap();

        assert_eq!(json.get("key1.key3").unwrap(), JsonValue::Bool(true));
    }

    #[test]
    fn test_set_deeply_nested_key() {
        let mut json = Json::from("{ \"key1\": { \"key2\": {} } }".to_string());

        json.set("key1.key2.key3.key4", JsonValue::String("deep_value".to_string()))
            .unwrap();

        assert_eq!(
            json.get("key1.key2.key3.key4").unwrap(),
            JsonValue::String("deep_value".to_string())
        );
    }

    #[test]
    fn test_set_creates_intermediate_objects() {
        let mut json = Json::from("{}".to_string());

        json.set("key1.key2.key3", JsonValue::Bool(false)).unwrap();

        assert_eq!(json.get("key1.key2.key3").unwrap(), JsonValue::Bool(false));
    }

    #[test]
    fn test_set_overwrites_existing_value() {
        let mut json = Json::from("{ \"key1\": { \"key2\": \"old_value\" } }".to_string());

        json.set("key1.key2", JsonValue::String("new_value".to_string()))
            .unwrap();

        assert_eq!(
            json.get("key1.key2").unwrap(),
            JsonValue::String("new_value".to_string())
        );
    }

    #[test]
    fn test_set_error_on_non_object_intermediate_key() {
        let mut json = Json::from("{ \"key1\": \"string_value\" }".to_string());

        let result = json.set("key1.key2", JsonValue::Bool(true));

        assert!(result.is_err());
    }


    #[test]
    fn test_set_creates_root_object_if_empty() {
        let mut json = Json::from("{}".to_string());

        json.set("key1", JsonValue::Bool(true)).unwrap();

        assert_eq!(json.get("key1").unwrap(), JsonValue::Bool(true));
    }

    #[test]
    fn test_set_handles_multiple_nested_keys() {
        let mut json = Json::from("{ \"key1\": { \"key2\": { \"key3\": \"value3\" } } }".to_string());

        json.set("key1.key2.key4", JsonValue::String("new_value".to_string()))
            .unwrap();

        assert_eq!(
            json.get("key1.key2.key4").unwrap(),
            JsonValue::String("new_value".to_string())
        );

        json.set("key1.key2.key5", JsonValue::Number(99)).unwrap();

        assert_eq!(json.get("key1.key2.key5").unwrap(), JsonValue::Number(99));
    }

    #[test]
    fn test_set_replaces_entire_object() {
        let mut json = Json::from("{ \"key1\": { \"key2\": { \"key3\": \"value3\" } } }".to_string());

        json.set(
            "key1.key2",
            JsonValue::Object(HashMap::from([(
                "new_key".to_string(),
                JsonValue::Bool(true),
            )])),
        )
            .unwrap();

        assert_eq!(
            json.get("key1.key2.new_key").unwrap(),
            JsonValue::Bool(true)
        );
        assert!(json.get("key1.key2.key3").is_err());
    }

    #[test]
    fn test_set_handles_empty_key() {
        let mut json = Json::from("{ \"key1\": { \"key2\": {} } }".to_string());

        let result = json.set("", JsonValue::Bool(true));
        assert!(result.is_err());
    }
}
