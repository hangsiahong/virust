use virust_runtime::struct_parser::StructParser;

#[test]
fn test_parse_simple_struct() {
    let code = r#"
    pub struct User {
        pub name: String,
        pub age: i32,
    }
    "#;

    let fields = StructParser::parse_struct(code, "User").unwrap();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "name");
    assert_eq!(fields[0].type_name, "String");
    assert!(!fields[0].is_optional);
    assert_eq!(fields[1].name, "age");
    assert_eq!(fields[1].type_name, "i32");
    assert!(!fields[1].is_optional);
}

#[test]
fn test_parse_with_optional_fields() {
    let code = r#"
    pub struct Product {
        pub id: String,
        pub description: Option<String>,
        pub price: Option<f64>,
    }
    "#;

    let fields = StructParser::parse_struct(code, "Product").unwrap();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].name, "id");
    assert!(!fields[0].is_optional);
    assert_eq!(fields[1].name, "description");
    assert!(fields[1].is_optional);
    assert_eq!(fields[2].name, "price");
    assert!(fields[2].is_optional);
}

#[test]
fn test_parse_complex_types() {
    let code = r#"
    pub struct Config {
        pub host: String,
        pub port: u16,
        pub timeout: Option<u64>,
    }
    "#;

    let fields = StructParser::parse_struct(code, "Config").unwrap();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].type_name, "String");
    assert_eq!(fields[1].type_name, "u16");
    assert_eq!(fields[2].type_name, "Option < u64 >");
    assert!(fields[2].is_optional);
}

#[test]
fn test_struct_name_mismatch() {
    let code = r#"
    pub struct User {
        pub name: String,
    }
    "#;

    let result = StructParser::parse_struct(code, "WrongName");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Struct name mismatch"));
}
