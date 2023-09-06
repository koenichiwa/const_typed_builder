use crate::field_info::{FieldInfo, FieldSettings};

pub struct StructInfo<'a> {
    pub name: &'a syn::Ident,
    pub field_infos: Vec<FieldInfo<'a>>,
    pub settings: StructSettings,
}

impl<'a> StructInfo<'a> {
    pub fn new(
        ast: &'a syn::DeriveInput,
        fields: &'a syn::FieldsNamed,
    ) -> Result<StructInfo<'a>, syn::Error> {
        let settings = StructSettings::default();
        let field_infos = fields
            .named
            .iter()
            .enumerate()
            .map(|(index, field)| FieldInfo::new(index, field, &settings.default_field_settings))
            .collect::<Result<_, _>>()?;
        Ok(StructInfo {
            name: &ast.ident,
            field_infos,
            settings,
        })
    }
}

pub struct StructSettings {
    default_field_settings: FieldSettings,
}

impl Default for StructSettings {
    fn default() -> Self {
        StructSettings {
            default_field_settings: FieldSettings::default(),
        }
    }
}

impl StructSettings {
    fn new() -> StructSettings {
        Default::default()
    }
}
