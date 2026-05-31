use std::collections::HashSet;

use super::{display, record::ModuleRecordCodegen};
use crate::{
    module::generics::{GenericKind, ModuleGenerics},
    shared::generics::GenericsHelper,
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Generics, parse_quote, Type};

/// Basic trait to be implemented for Module generation.
pub(crate) trait ModuleCodegen {
    type RecordCodegen: ModuleRecordCodegen;

    fn gen_num_params(&self) -> TokenStream;
    fn gen_visit(&self) -> TokenStream;
    fn gen_collect_devices(&self) -> TokenStream;
    fn gen_to_device(&self) -> TokenStream;
    fn gen_fork(&self) -> TokenStream;
    fn gen_map(&self) -> TokenStream;
    fn gen_valid(&self) -> TokenStream;
    fn gen_from_inner(&self) -> TokenStream;
    fn gen_into_record(&self) -> TokenStream;
    fn gen_load_record(&self) -> TokenStream;
    fn gen_clone(&self) -> TokenStream;

    fn record_codegen(self) -> Self::RecordCodegen;

    fn gen_display(&self) -> TokenStream;

    fn module_generics(&self) -> &ModuleGenerics;
}

pub(crate) fn generate_module_standard<Codegen: ModuleCodegen>(
    ast: &syn::DeriveInput,
    codegen: Codegen,
) -> TokenStream {
    let name = &ast.ident;

    let generics = GenericsParser::from_ast(&ast.generics, codegen.module_generics());

    let display_fn = display::display_fn(ast);
    let attributes_fn = codegen.gen_display();
    let num_params_fn = codegen.gen_num_params();
    let visit = codegen.gen_visit();
    let map_mut = codegen.gen_map();
    let collect_devices = codegen.gen_collect_devices();
    let to_device = codegen.gen_to_device();
    let fork = codegen.gen_fork();
    let valid_fn = codegen.gen_valid();
    let from_inner_fn = codegen.gen_from_inner();
    let into_record_fn = codegen.gen_into_record();
    let load_record_fn = codegen.gen_load_record();
    let clone_fn = codegen.gen_clone();

    let record = codegen.record_codegen();
    let record_name = Ident::new(format!("{name}Record").as_str(), name.span());
    let (record_type, record_generics) = record.gen_record_type(&record_name, &generics.module);

    let (generics_module, generics_ty_module, generics_where_module) =
        generics.module.split_for_impl();
    let (generics_module_autodiff, generics_ty_module_autodiff, generics_where_module_autodiff) =
        generics.module_autodiff.split_for_impl();
    let (_, generics_ty_record, _) = record_generics.split_for_impl();

    let mut codegen = quote! {

        impl #generics_module burn::module::Module for #name #generics_ty_module #generics_where_module {
            type Record = #record_name #generics_ty_record;

            #load_record_fn
            #into_record_fn

            #num_params_fn

            #visit
            #map_mut

            #collect_devices
            #to_device
            #fork

        }

        impl #generics_module_autodiff burn::module::AutodiffModule for #name #generics_ty_module_autodiff #generics_where_module_autodiff
        {
            #valid_fn

            #from_inner_fn
        }

        impl #generics_module core::fmt::Display for #name #generics_ty_module #generics_where_module {
            #display_fn
        }

        impl #generics_module burn::module::ModuleDisplayDefault for #name #generics_ty_module #generics_where_module {
            #attributes_fn

            fn num_params(&self) -> usize {
                burn::module::Module::num_params(self)
            }
        }

        impl #generics_module Clone for #name #generics_ty_module #generics_where_module {
            #clone_fn
        }

        #record_type
    };

    if !has_custom_display(&ast.attrs) {
        codegen.extend(quote! {
            impl #generics_module burn::module::ModuleDisplay for #name #generics_ty_module #generics_where_module {

            }
        });
    }

    codegen
}

// When there is inner param or module, the type is considered stateless.
pub(crate) fn generate_module_stateless<Codegen: ModuleCodegen>(
    ast: &syn::DeriveInput,
    codegen: Codegen, // Pass the codegen here
) -> TokenStream {
    let name = &ast.ident;
    let (generics, generics_ty, generics_where) = ast.generics.split_for_impl();

    let display_fn = display::display_fn(ast);
    let attributes_fn = codegen.gen_display(); // Use codegen for attributes too
    let clone_fn = codegen.gen_clone(); // The automatic clone logic

    let mut codegen = quote! {
        impl #generics burn::module::Module for #name #generics_ty #generics_where {
            burn::empty!(module);
        }

        impl #generics burn::module::AutodiffModule for #name #generics_ty #generics_where {
            burn::empty!(ad_module, #name #generics_ty);
        }

        impl #generics core::fmt::Display for #name #generics_ty #generics_where {
            #display_fn
        }

        impl #generics burn::module::ModuleDisplayDefault for #name #generics_ty #generics_where {
            #attributes_fn
        }

        impl #generics Clone for #name #generics_ty #generics_where {
            #clone_fn
        }
    };

    if !has_custom_display(&ast.attrs) {
        codegen.extend(quote! {
            impl  #generics burn::module::ModuleDisplay for #name #generics_ty #generics_where {

            }
        });
    }

    codegen
}

struct GenericsParser {
    module: Generics,
    module_autodiff: Generics,
}

impl GenericsParser {
    fn from_ast(generics: &Generics, module_generics: &ModuleGenerics) -> Self {
        let mut module = GenericsHelper::new(generics.clone());
        let mut module_autodiff = GenericsHelper::new(generics.clone());

        module.types().into_iter().for_each(|ident| {
            // By default, require module bound
            let mut requires_module_bound = true;
            let mut generic_kind = None;
            if !module_generics.is_empty() {
                generic_kind = module_generics.get_generic_kind(&ident);
                let has_module_bound = matches!(generic_kind, Some(GenericKind::Module));
                let is_unbounded = matches!(generic_kind, Some(GenericKind::Plain));

                requires_module_bound = has_module_bound || is_unbounded;
            }

            if requires_module_bound {
                module.add_predicate(parse_quote! {
                    #ident: burn::module::Module
                });

                module.add_predicate(parse_quote! {
                    #ident: burn::module::ModuleDisplay
                });

                module_autodiff.add_predicate(parse_quote! {
                    #ident: burn::module::AutodiffModule
                });

                module_autodiff.add_predicate(parse_quote! {
                    #ident: burn::module::ModuleDisplay
                });
            } else {
                // Add required bounds to impl
                if let Some(GenericKind::Skip) = generic_kind {
                    module.add_predicate(parse_quote! {
                        #ident: Clone + core::fmt::Debug + Send
                    });
                    module_autodiff.add_predicate(parse_quote! {
                        #ident: Clone + core::fmt::Debug + Send
                    });
                }
            }
        });

        Self {
            module: module.generics,
            module_autodiff: module_autodiff.generics,
        }
    }
}

#[derive(Debug)]
pub enum ModuleFieldAttribute {
    Skip,
}

#[derive(Default, Debug)]
pub struct ModuleFieldType {
    pub is_module: bool,
    pub attr: Option<ModuleFieldAttribute>,
    pub generic_idents: HashSet<Ident>,
}

impl ModuleFieldType {
    /// Returns true if the field is a module with parameters
    /// (i.e., a real module that is neither skipped nor constant).
    pub fn is_parameter_module(&self) -> bool {
        self.is_module && self.attr.is_none()
    }

    /// Returns true for modules that should be persisted, including constants.
    pub fn is_persistent_module(&self) -> bool {
        self.is_module && !matches!(self.attr, Some(ModuleFieldAttribute::Skip))
    }

    /// Returns true for generic fields that are assumed to be modules.
    pub fn maybe_generic_module(&self) -> bool {
        // We assumed it might be a module generic if the field is not marked
        // by any attributes (skip or constant)
        !self.generic_idents.is_empty() && self.attr.is_none()
    }
}

fn has_custom_display(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("module")
            && attr
                .parse_nested_meta(|meta| {
                    if meta.path.is_ident("custom_display") {
                        Ok(())
                    } else {
                        Err(meta.error("unsupported attribute"))
                    }
                })
                .is_ok()
    })
}

/// Returns an error if any module field has a generic type of the
/// `GenericKind::Skip` kind, because such a field should not be skipped.
pub(crate) fn check_skipped_module_generic<'a>(
    fields: impl Iterator<Item = (&'a Type, &'a ModuleFieldType)>,
    generics: &ModuleGenerics,
) -> syn::Result<()> {
    // Multiple errors are combined into one
    fields
        .filter_map(|(ty, mft)| {
            // TODO: comment
            if mft.is_module
                && let Type::Path(p) = ty
                && let Some(id) = p.path.get_ident()
                && mft.generic_idents.contains(id)
            {
                debug_assert_eq!(mft.generic_idents.len(), 1);
                let (generic, kind) = generics.get_entry(id).expect(
                    "Generic used in a field should be one of the generics declared by its parent type"
                );
                if matches!(kind, GenericKind::Skip) {
                    return Some(syn::Error::new(
                        generic.span(),
                        "Generic type should not be used on a module field and \
                         a skipped field at the same time",
                    ));
                }
            }
            None
        })
        .reduce(|mut sink, err| { sink.combine(err); sink })
        .map_or(Ok(()), |sink| Err(sink))
}
