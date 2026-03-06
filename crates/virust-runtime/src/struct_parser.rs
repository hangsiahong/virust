use syn::{parse_str, DeriveInput, Data, DataStruct};
use quote::ToTokens;

#[derive(Debug)]
pub struct FieldInfo {
    pub name: String,
    pub type_name: String,
    pub is_optional: bool,
}

pub struct StructParser;

impl StructParser {
    pub fn parse_struct(code: &str, struct_name: &str) -> Result<Vec<FieldInfo>, String> {
        let ast: DeriveInput = parse_str(code)
            .map_err(|e| format!("Failed to parse: {}", e))?;

        if ast.ident != struct_name {
            return Err(format!("Struct name mismatch: expected {}, found {}",
                struct_name, ast.ident));
        }

        let fields = match &ast.data {
            Data::Struct(DataStruct { fields, .. }) => fields,
            _ => return Err("Not a struct".to_string()),
        };

        let mut field_infos = Vec::new();
        for field in fields {
            if let Some(ident) = &field.ident {
                let type_name = field.ty.to_token_stream().to_string();
                let is_optional = type_name.replace(" ", "").contains("Option<");

                field_infos.push(FieldInfo {
                    name: ident.to_string(),
                    type_name,
                    is_optional,
                });
            }
        }

        Ok(field_infos)
    }
}
