mod field_generator;
mod group_generator;
mod data_generator;
mod target_generator;
mod builder_generator;
use quote::quote;

use crate::{struct_info::StructInfo, StreamResult};

use self::{field_generator::FieldGenerator, group_generator::GroupGenerator, data_generator::DataGenerator, target_generator::TargetGenerator, builder_generator::BuilderGenerator};

pub struct Generator<'a> {
    data_gen: DataGenerator<'a>,
    target_gen: TargetGenerator<'a>,
    builder_gen: BuilderGenerator<'a>,
}

impl<'a> Generator<'a> {
    pub fn new(info: &'a StructInfo<'a>) -> Self {
        let field_gen = FieldGenerator::new(&info.field_infos());
        let group_gen = GroupGenerator::new(info.groups().values().collect());
        Generator {
            data_gen: DataGenerator::new(field_gen.clone(), info.name(), info.data_name() ),
            target_gen: TargetGenerator::new(field_gen.clone(), info.name(), info.builder_name()),
            builder_gen: BuilderGenerator::new(group_gen, field_gen, info.name(), info.builder_name(), info.data_name()) 
        }
    }

    pub fn generate(&self) -> StreamResult {
        let target = self.target_gen.generate();
        let data = self.data_gen.generate()?;
        let builder = self.builder_gen.generate()?;
        let tokens = quote!(
            #target
            #builder
            #data
        );
        Ok(tokens)
    }
}