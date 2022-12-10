mod derive;

use derive::{titled_routable_derive_impl, TitledRoutable};
use syn::parse_macro_input;

#[proc_macro_derive(TitledRoutable, attributes(title))]
pub fn routable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as TitledRoutable);
    titled_routable_derive_impl(input).into()
}
