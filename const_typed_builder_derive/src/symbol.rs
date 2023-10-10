use strum::{AsRefStr, Display, EnumString};

#[derive(AsRefStr, Debug, Display, EnumString, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Symbol {
    Builder,
    Group,
    Mandatory,
    Skip,
    Optional,
    AssumeMandatory,

    Single,
    AtLeast,
    AtMost,
    Exact,

    Solver,
    BruteForce,
    Compiler,

    Propagate,
    Into,
    #[strum(serialize = "as_ref", serialize = "asref")]
    AsRef,
    #[strum(serialize = "as_mut", serialize = "asmut")]
    AsMut,
    Standard,
}
