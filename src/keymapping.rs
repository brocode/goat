use std::collections::BTreeMap;

pub struct KeyMapping {
  pub ret_code: i32,
  pub label: String,
}

pub fn parse_mappings(raw_mappings: Vec<String>) -> Result<BTreeMap<char, KeyMapping>, String> {
  let mut mappings: BTreeMap<char, KeyMapping> = BTreeMap::new();
  for mapping in raw_mappings {
    let mut split: Vec<&str> = mapping.split(':').collect();
    if split.len() == 3 {
      if let Some(char) = split[1].chars().next() {
        if let Ok(ret_code) = split[0].parse::<i32>() {
          if ret_code > 113 || ret_code < 64 {
            return Err(format!(
              "Invalid mapping '{}', retcode should be < 64 or > 113",
              mapping
            ));
          }
          mappings.insert(
            char,
            KeyMapping {
              ret_code,
              label: split[2].to_string(),
            },
          );
        } else {
          return Err(format!(
            "Invalid mapping '{}', retcode should be a number",
            mapping
          ));
        }
      } else {
        return Err(format!(
          "Invalid mapping '{}', keycode should be a char",
          mapping
        ));
      }
    } else {
      return Err(format!(
        "Invalid mapping '{}', format should be <retcode>:<key>:<label>",
        mapping
      ));
    }
  }
  mappings.insert(
    'q',
    KeyMapping {
      ret_code: 1,
      label: "abort".to_string(),
    },
  );
  mappings.insert(
    'c',
    KeyMapping {
      ret_code: 0,
      label: "continue".to_string(),
    },
  );
  Ok(mappings)
}

#[test]
fn test_parse_valid_mapping() {
  let mapping = parse_mappings(vec!["65:a:fkbr".to_string()]).unwrap();
  assert!(mapping.contains_key(&'a'));
  let key_mapping = mapping.get(&'a').unwrap();
  assert_eq!(key_mapping.label, "fkbr");
  assert_eq!(key_mapping.ret_code, 65);
}

#[test]
fn test_parse_invalid_mapping() {
  assert!(parse_mappings(vec!["65a:fkbr".to_string()]).is_err());
  assert!(parse_mappings(vec!["63:a:fkbr".to_string()]).is_err());
  assert!(parse_mappings(vec!["b:a:fkbr".to_string()]).is_err());
}
