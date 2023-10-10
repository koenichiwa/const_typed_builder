use strum::{AsRefStr, Display, EnumString};

#[derive(AsRefStr, Debug, Display, EnumString, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Symbol {
    // Top level attributes
    Builder,
    Groups,
    
    // Field kinds
    Group, // Deprecated as top level attribute
    Mandatory,
    Skip,
    Optional,
    AssumeMandatory,

    // Group kinds
    Single,
    AtLeast,
    AtMost,
    Exact,

    // Solver kinds
    Solver,
    BruteForce,
    Compiler,

    // Setter kinds
    Propagate,
    Into,
    #[strum(serialize = "as_ref", serialize = "asref")]
    AsRef,
    #[strum(serialize = "as_mut", serialize = "asmut")]
    AsMut,
    Standard,
}
