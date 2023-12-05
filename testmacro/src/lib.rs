extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn test_item(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let name = &input.sig.ident.to_string();
    let func = &input.block;

    let r = quote! {
        #[test_case]
        const t: TestType = TestType {
            modname: module_path!(),
            name: #name,
            f: || -> Result<(),()> {
                #func
                Ok(())
            }
        };
    };
    r.into()
}
