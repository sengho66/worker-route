#[derive(Debug)]
pub struct Error(syn::Error);

impl Error {
    pub fn inner(self) -> syn::Error {
        self.0
    }
}

impl From<syn::Error> for Error {
    fn from(value: syn::Error) -> Self {
        Self(value)
    }
}
