pub fn is_option(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if type_path.qself.is_some() {
            return false;
        }
        if let Some(segment) = type_path.path.segments.last() {
            segment.ident == syn::Ident::new("Option", segment.ident.span())
        } else {
            false
        }
    } else {
        false
    }
}

pub fn inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let path = if let syn::Type::Path(type_path) = ty {
        if type_path.qself.is_some() {
            return None;
        }
        &type_path.path
    } else {
        return None;
    };
    let segment = path.segments.last()?;
    let syn::PathArguments::AngleBracketed(generic_params) = &segment.arguments else {
        return None;
    };
    
    if let syn::GenericArgument::Type(inner) = generic_params.args.first()? {
        Some(inner)
    } else {
        None
    }
}
