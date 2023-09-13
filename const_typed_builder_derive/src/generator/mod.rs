mod builder_generator;
mod data_generator;
mod field_generator;
mod group_generator;
mod target_generator;

use quote::quote;
use crate::{info::StructInfo, StreamResult};
use self::{
    builder_generator::BuilderGenerator, data_generator::DataGenerator,
    field_generator::FieldGenerator, group_generator::GroupGenerator,
    target_generator::TargetGenerator,
};

pub struct Generator<'a> {
    data_gen: DataGenerator<'a>,
    target_gen: TargetGenerator<'a>,
    builder_gen: BuilderGenerator<'a>,
}

impl<'a> Generator<'a> {
    pub fn new(info: &'a StructInfo<'a>) -> Self {
        let field_gen = FieldGenerator::new(&info.field_infos(), info.generics());
        let group_gen = GroupGenerator::new(info.groups().values().collect());
        Generator {
            data_gen: DataGenerator::new(field_gen.clone(), info.name(), info.data_name()),
            target_gen: TargetGenerator::new(field_gen.clone(), info.name(), info.builder_name()),
            builder_gen: BuilderGenerator::new(
                group_gen,
                field_gen,
                info.name(),
                info.vis(),
                info.generics(),
                info.builder_name(),
                info.data_name(),
            ),
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
