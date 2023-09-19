mod builder_generator;
mod data_generator;
mod field_generator;
mod generics_generator;
mod group_generator;
mod target_generator;

use self::{
    builder_generator::BuilderGenerator, data_generator::DataGenerator,
    field_generator::FieldGenerator, generics_generator::GenericsGenerator,
    group_generator::GroupGenerator, target_generator::TargetGenerator,
};
use crate::{info::StructInfo, StreamResult};
use quote::quote;

/// The `Generator` struct is responsible for generating code for the builder pattern based on the provided `StructInfo`.
pub struct Generator<'a> {
    data_gen: DataGenerator<'a>,
    target_gen: TargetGenerator<'a>,
    builder_gen: BuilderGenerator<'a>,
}

impl<'a> Generator<'a> {
    /// Creates a new `Generator` instance for code generation.
    ///
    /// # Arguments
    ///
    /// - `info`: A reference to the `StructInfo` representing the input struct.
    ///
    /// # Returns
    ///
    /// A `Generator` instance initialized with the provided `StructInfo`.
    pub fn new(info: &'a StructInfo<'a>) -> Self {
        let generics_gen = GenericsGenerator::new(info.field_infos(), info.generics());
        let field_gen = FieldGenerator::new(info.field_infos());
        let group_gen = GroupGenerator::new(info.groups().values().collect());
        Generator {
            data_gen: DataGenerator::new(
                field_gen.clone(),
                generics_gen.clone(),
                info.name(),
                info.data_name(),
            ),
            target_gen: TargetGenerator::new(
                generics_gen.clone(),
                info.name(),
                info.builder_name(),
            ),
            builder_gen: BuilderGenerator::new(
                group_gen,
                field_gen,
                generics_gen,
                info.name(),
                info.vis(),
                info.builder_name(),
                info.data_name(),
                info.solve_type(),
            ),
        }
    }

    /// Generates the builder pattern code and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `StreamResult` representing the generated token stream.
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
