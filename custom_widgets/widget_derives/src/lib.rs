use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields};






#[proc_macro_derive(MenuBarOption)]
pub fn derive_menu_bar_option(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    // Used in the quasi-quotation below as `#name`.


    let all_top_variants = get_all_top_variants(&input);

    let menu_options = derive_menu_option(&input);

    let menu_bar_options = derive_menubar_option(&input);




    // Generate an expression to sum up the heap size of each field.

    let expanded = quote! {
        #all_top_variants

        #menu_bar_options

        #menu_options

    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)


}


#[proc_macro_derive(MenuOption)]
///this macro doesn't derive the MenuOption trait, but derives helpers like ALL
pub fn widget_menu(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    // Used in the quasi-quotation below as `#name`.

    let data = &input.data;
    let menu_name = &input.ident;
    let len = if let Data::Enum(ref fields) = data {
        fields.variants.len()
    } else {
        unimplemented!("Only enum types are supported for MenuOption")
    };



    let all_content = if let Data::Enum(ref fields) = data {
           fields.variants.iter().map(|f| {
                let field_name = &f.ident;
                quote_spanned!{ f.span()=>
                    &#menu_name::#field_name,
                }

            })
        }
        else {
            unimplemented!("Only enum types are supported for MenuOption")
        }
    ;

    let first_enum = if let Data::Enum(ref fields) = data {
           fields.variants.first().map(|f| {
                let field_name = &f.ident;
                quote!{ 
                     #menu_name::#field_name
                }

            })
        }
        else {
            unimplemented!("Only enum types are supported for MenuOption")
        }
    ;





    // Generate an expression to sum up the heap size of each field.

    let expanded = quote! {
        impl #menu_name {
            const ALL : [&'static dyn MenuOption<Message=<Self as MenuOption>::Message>; #len] =  [ #(#all_content) *];
            const fn base() -> #menu_name { #first_enum }
        }
       
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)


}







fn derive_menubar_option(input: &DeriveInput) -> TokenStream {
    let data = &input.data;
    let top_menu_name = &input.ident;



    match *data {
        Data::Enum(ref fields) => {
           let get_children_content =  fields.variants.iter().map(|f| {
               let field_name = &f.ident;
                
            
               quote_spanned!{ f.span()=>
                   #top_menu_name::#field_name(default) => {default.all()},
               }

           });
           quote! { 
                impl MenuBarOption for #top_menu_name {
                    type Message = #top_menu_name;

                    fn all() -> &'static [Self] {
                        &Self::ALL
                    }
                    fn get_children(&self) -> &'static [&dyn MenuOption<Message=#top_menu_name>] {
                        match self {
                            #(
                                #get_children_content
                            ) *
                        }
                    }
                }
            }
        }
        _ => unimplemented!("TopMenu only supports enum types")
    }




}




fn get_all_top_variants(input: &DeriveInput) -> TokenStream {
    let data = &input.data;
    let top_menu_name = &input.ident;



    match *data {
        Data::Enum(ref fields) => {
           let length = fields.variants.len();
           let const_all_content =  fields.variants.iter().map(|f| {
               let field_name = &f.ident;
                let type_name = match f.fields {
                    Fields::Unnamed(ref field_content) => {
                        field_content
                            .unnamed
                            .first()
                            .expect("TopMenuType expects at least one inner element!")
                            .ty
                            .clone()
                    }
                    _ => panic!("Poorly formed type! Each TopMenu variant must have one Unnamed tuple")
                };


              
               quote_spanned!{ f.span()=>
                   #top_menu_name::#field_name(#type_name::base()), 
               }

           });
           quote! { 
            impl #top_menu_name {
               const ALL: [#top_menu_name; #length] = [#(#const_all_content) *];

            }
           }
        }
        _ => unimplemented!("TopMenu only supports enum types")
    }



}


fn derive_menu_option(input: &DeriveInput) -> TokenStream {
    let data = &input.data;
    let top_menu_name = &input.ident;
    
    match *data {
        Data::Enum(ref fields) => {
            let all_variants = fields.variants.iter().map(|f| {
                let name = &f.ident;
                let type_name = match f.fields {
                    Fields::Unnamed(ref field_content) => {
                        field_content
                            .unnamed
                            .first()
                            .expect("TopMenuType expects at least one inner element!")
                            .ty
                            .clone()
                    }
                    _ => unimplemented!("Poorly formed type! Only unnamed fields allowed for TopMenuType")
                };


                quote_spanned! { f.span()=>
                    impl MenuOption for #type_name {
                        type Message = #top_menu_name;

                        fn to_message(&self) -> Self::Message {
                            #top_menu_name::#name(self.clone())
                        }

                        fn all(&self) -> &'static [&dyn MenuOption<Message=Self::Message>] {
                            &#type_name::ALL
                        }


                    }
                }
            });
            quote! {
                #( #all_variants ) 

                *
            }
        }
        _ => unimplemented!("Only enum types are acceptable for deriving MenuBarOption!")
    }

}
