use serde_json::value::Value;

pub fn parse_arguments(arguments: String) -> Value {
    parse_map(arguments.clone())
}

fn parse_value(value: String) -> Value {
    let mut is_map = false;
    let mut is_list = false;

    let mut characters = value.chars();
    let mut character = characters.next();
    let mut depth: usize = 0;

    while character.is_some() {
        match character {
            Some('{') => {
                if depth == 0 {
                    is_map = true;
                    is_list = true;
                }
                depth += 1;
            }
            Some('}') => depth -= 1,
            Some('=') => {
                if depth == 1 {
                    is_list = false;
                }
            }
            _ => {}
        }

        character = characters.next();
    }

    if is_list {
        return parse_list(value);
    } else if is_map {
        return parse_map(value);
    } else {
        // Check if the value is a number
        if let Ok(number) = value.parse::<i64>() {
            return serde_json::Number::from(number).into();
        }
        // Check if the value is a float
        if let Ok(number) = value.parse::<f64>() {
            return serde_json::Number::from_f64(number).into();
        }
        return Value::String(value);
    }
}

fn parse_map(arguments: String) -> Value {
    let mut parsed_arguments = serde_json::Map::new();

    let mut current: String = String::new();
    let mut current_key: String = String::new();
    let mut current_value: String;

    // Split the arguments into key-value pairs
    let mut splitted_kv_pairs = Vec::<(String, String)>::new();

    let mut characters = match arguments.chars().nth(0) {
        Some('{') => arguments.chars().skip(1),
        _ => arguments.chars().skip(0),
    };
    let mut character = characters.next();

    let mut depth: usize = 0;
    while character.is_some() {
        match character {
            Some('{') => depth += 1,
            Some('}') => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            Some('=') => {
                if depth == 0 {
                    current_key = current.trim().to_string();
                    current = String::new();

                    character = characters.next();
                    continue;
                }
            }
            Some('\n') => {
                if depth == 0 && !current_key.is_empty() {
                    current_value = current;
                    current = String::new();

                    splitted_kv_pairs.push((current_key, current_value));
                    current_key = String::new();

                    character = characters.next();
                    continue;
                }
            }
            _ => {}
        }

        current.push(character.unwrap());
        character = characters.next();
    }

    for (key, value) in splitted_kv_pairs {
        parsed_arguments.insert(key, parse_value(value));
    }

    serde_json::Value::Object(parsed_arguments)
}

fn parse_list(arguments: String) -> Value {
    let mut parsed_arguments = Vec::<Value>::new();

    let mut current: String = String::new();

    // Split the arguments
    let mut splitted_values = Vec::<String>::new();

    let mut characters = arguments.chars().skip(1);
    let mut character = characters.next();

    let mut depth: usize = 0;
    while character.is_some() {
        match character {
            Some('{') => depth += 1,
            Some('}') => {
                if depth == 0 {
                    splitted_values.push(current.trim().to_string());
                    break;
                }
                depth -= 1;
            }
            Some(',') => {
                if depth == 0 {
                    splitted_values.push(current.trim().to_string());
                    current = String::new();

                    character = characters.next();
                    continue;
                }
            }
            _ => {}
        }

        current.push(character.unwrap());
        character = characters.next();
    }

    for value in splitted_values {
        parsed_arguments.push(parse_value(value));
    }

    serde_json::Value::Array(parsed_arguments)
}
