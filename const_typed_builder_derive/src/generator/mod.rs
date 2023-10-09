mod builder_generator;
mod data_generator;
mod target_generator;

use self::{
    builder_generator::BuilderGenerator, data_generator::DataGenerator,
    target_generator::TargetGenerator,
};
use crate::info;
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
    /// - `info`: The `info::Container` containing all the information of the data container.
    ///
    /// # Returns
    ///
    /// A `Generator` instance initialized with the provided `StructInfo`.
    pub fn new(info: &'info info::Container<'info>) -> Self {
        info.group_collection()
            .values()
            .for_each(|group| group.check());

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
    use proc_macro2::TokenStream;
    use quote::quote;

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
    pub fn const_generics_all_valued(
        value: bool,
        fields: &info::FieldCollection,
        generics: &syn::Generics,
    ) -> TokenStream {
        let mut all = fields
            .iter()
            .filter_map(info::TrackedField::new)
            .map(|_| quote!(#value));
        add_const_valued_generics_for_type(&mut all, generics)
    }

    /// Adds valued const generics to the target structs `syn::Generics` and returns a `Tokenstream` instance.
    ///
    /// # Returns
    ///
    /// A `Tokenstream` instance representing the generics for the builder struct.
    pub fn add_const_valued_generics_for_type(
        constants: &mut impl Iterator<Item = TokenStream>,
        generics: &syn::Generics,
    ) -> TokenStream {
        dbg!(generics.type_params().collect::<Vec<_>>());
        let generic_idents = generics.params.iter().map(|param| match param {
            syn::GenericParam::Lifetime(lt) => &lt.lifetime.ident,
            syn::GenericParam::Type(ty) => &ty.ident,
            syn::GenericParam::Const(cnst) => &cnst.ident,
        });

        quote!(< #(#generic_idents,)* #(#constants),* >)
    }
}
