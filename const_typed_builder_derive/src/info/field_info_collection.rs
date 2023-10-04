use std::collections::BTreeMap;

use super::FieldInfo;
#[derive(Debug)]
pub enum FieldInfoCollection<'a> {
    StructFields {
        fields: Vec<FieldInfo<'a>>,
    },
    EnumFields {
        variant_fields: BTreeMap<syn::Ident, Vec<FieldInfo<'a>>>,
    },
}

impl<'a> FieldInfoCollection<'a> {
    pub fn all_fields(&self) -> Vec<&FieldInfo<'a>> {
        match self {
            FieldInfoCollection::StructFields { fields } => fields.iter().collect(),
            FieldInfoCollection::EnumFields { variant_fields } => {
                variant_fields.values().flatten().collect()
            }
        }
    }

    pub fn fields_by_variant_ident(&self, variant: syn::Ident) -> Option<&Vec<FieldInfo>> {
        // TODO: Change to Error
        match self {
            FieldInfoCollection::StructFields { fields } => None,
            FieldInfoCollection::EnumFields { variant_fields } => variant_fields.get(&variant),
        }
    }

    pub fn variant_idents(&self) -> Option<impl Iterator<Item = &syn::Ident>> {
        match self {
            FieldInfoCollection::StructFields { .. } => None,
            FieldInfoCollection::EnumFields { variant_fields } => Some(variant_fields.keys()),
        }
    }

    pub fn variant_count(&self) -> usize {
        match self {
            FieldInfoCollection::StructFields { fields } => 0,
            FieldInfoCollection::EnumFields { variant_fields } => variant_fields.len(),
        }
    }
}
