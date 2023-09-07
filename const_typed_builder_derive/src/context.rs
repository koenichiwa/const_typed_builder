pub struct Context {
    error: Option<syn::Error>,
}

impl Context {
    pub fn new() -> Self {
        Context { error: None }
    }
}
