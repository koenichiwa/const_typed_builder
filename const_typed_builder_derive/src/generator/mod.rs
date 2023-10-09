mod builder_generator;
mod data_generator;
mod target_generator;

use self::{
    builder_generator::BuilderGenerator, data_generator::DataGenerator,
    target_generator::TargetGenerator,
};
use crate::info::{Container, self};
use proc_macro2::TokenStream;
use quote::quote;

/// The `Generator` struct is responsible for generating code for the builder pattern based on the provided `StructInfo`.
pub struct Generator<'info> {
    info: &'info info::Container<'info>,
    data_gen: DataGenerator<'info>,
    target_gen: TargetGenerator<'info>,
    builder_gen: BuilderGenerator<'info>,
}

impl<'info> Generator<'info> {
    /// Creates a new `Generator` instance for code generation.
    ///
    /// # Arguments
    ///
    /// - `info`: A reference to the `StructInfo` representing the input struct.
    ///
    /// # Returns
    ///
    /// A `Generator` instance initialized with the provided `StructInfo`.
    pub fn new(info: &'info Container<'info>) -> Self {
        info.groups().values().for_each(|group| group.check());

        Generator {
            info,
            data_gen: DataGenerator::new(info),
            target_gen: TargetGenerator::new(info),
            builder_gen: BuilderGenerator::new(info),
        }
    }

    /// Generates the builder pattern code and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `TokenStream` representing the generated token stream.
    pub fn generate(&self) -> TokenStream {
        let target = self.target_gen.generate();
        let data = self.data_gen.generate();
        let builder = self.builder_gen.generate();

        if self.info.generate_module() {
            let mod_ident = self.info.mod_ident();
            let target_ident = self.info.ident();
            quote!(
                #target
                mod #mod_ident {
                    use super::#target_ident;
                    #builder
                    #data
                }
            )
        } else {
            quote!(
                #target
                #builder
                #data
            )
        }
    }
}

mod util {
    use either::Either;
    use proc_macro2::TokenStream;
    use quote::{quote, ToTokens};

    use crate::info;
    /// Generates const generics with boolean values and returns a token stream.
    ///
    /// # Arguments
    ///
    /// - `value`: A boolean value to set for the const generics.
    ///
    /// # Returns
    ///
    /// A `TokenStream` representing the generated const generics.
    pub fn const_generics_all_valued(value: bool, fields: &info::FieldCollection, generics: &syn::Generics) -> TokenStream {
        let mut all = fields
            .iter()
            .filter_map(|field| match field.kind() {
                info::FieldKind::Skipped | info::FieldKind::Optional => None,
                info::FieldKind::Mandatory | info::FieldKind::Grouped => Some(Either::Right(
                    syn::LitBool::new(value, field.ident().span()),
                )),
            });
        add_const_valued_generics_for_type(&mut all, generics)
    }

    /// Adds valued const generics to the target structs `syn::Generics` and returns a `Tokenstream` instance.
    ///
    /// # Returns
    ///
    /// A `Tokenstream` instance representing the generics for the builder struct.
    pub fn add_const_valued_generics_for_type(
        constants: &mut dyn Iterator<Item = Either<syn::Ident, syn::LitBool>>,
        generics: &syn::Generics,
    ) -> TokenStream {
        let type_generics = generics.params.iter().map(|param| match param {
            syn::GenericParam::Lifetime(lt) => &lt.lifetime.ident,
            syn::GenericParam::Type(ty) => &ty.ident,
            syn::GenericParam::Const(cnst) => &cnst.ident,
        });

        let tokens = constants.map(|constant| {
            constant
                .map_either(|iden| iden.to_token_stream(), |lit| lit.to_token_stream())
                .into_inner()
        });
        quote!(< #(#type_generics,)* #(#tokens),* >)
    }
}
