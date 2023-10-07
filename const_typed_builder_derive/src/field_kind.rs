#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FieldKind {
    Optional,
    Skipped,
    Mandatory,
    Grouped,
}
