use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
enum JsonRoot {
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Json {
    data: JsonRoot,
}

impl From<String> for Json {
    fn from(data: String) -> Json {
        let parsed = JsonValue::parse(&data).unwrap(); // Парсинг строки
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

#[derive(Debug, Eq, PartialEq)]
enum JsonValue {
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
    String(String),
    Number(i64),
    Bool(bool),
    Null,
}

impl JsonValue {
    pub fn parse(input: &str) -> Result<JsonValue, String> {
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

    fn parse_object(input: &str) -> Result<JsonValue, String> {
        let input = input.trim();
        if !input.starts_with('{') || !input.ends_with('}') {
            return Err("Invalid JSON object: must start with '{' and end with '}'".to_string());
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
                        return Err(format!("Invalid JSON object: {}", ch));
                    }
                    if is_key {
                        current_key.push(ch);
                    } else {
                        current_value.push(ch);
                    }
                }
                ',' if stack.is_empty() && !inside_string => {
                    if elements.insert(
                        current_key.clone(),
                        Self::parse(&current_value)?,
                    ).is_some() {panic!("Invalid JSON object")};
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
            if elements.insert(
                current_key.clone(),
                Self::parse(&current_value)?,
            ).is_some() {panic!("Invalid JSON object")};
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
            return Err(format!("Invalid JSON object: {}", input));
        }

        Ok(JsonValue::Object(elements))
    }

    fn parse_array(input: &str) -> Result<JsonValue, String> {
        let input = input.trim();
        if !input.starts_with('[') || !input.ends_with(']') {
            return Err("Invalid JSON array: must start with '[' and end with ']'".to_string());
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
                        return Err(format!("Invalid JSON array: {}", ch));
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
            Err(format!("Invalid JSON array: {}", current))
        } else if !stack.is_empty() {
            Err(format!("Invalid JSON array: {:?}", stack))
        } else {
            Ok(JsonValue::Array(elements))
        }
    }

    fn parse_string(input: &str) -> Result<JsonValue, String> {
        Ok(JsonValue::String(input.trim_matches('"').to_string()))
    }

    fn parse_number(input: &str) -> Result<JsonValue, String> {
        Ok(input.parse::<i64>().map(JsonValue::Number).unwrap())
    }

    fn parse_bool(input: &str) -> Result<JsonValue, String> {
        Ok(JsonValue::Bool(input == "true"))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

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

        // Ожидаемое значение
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
}
