use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use std::{collections::HashSet, format, string::ToString};
use syn::{
    DeriveInput, Expr, ExprCall, ExprPath, Ident, LitStr, Path, Result, parse::Parse,
    punctuated::Punctuated, spanned::Spanned, token::Comma,
};

use super::{
    relationship::{RELATIONSHIP, RELATIONSHIP_TARGET, Relationship, RelationshipTarget},
    require::Require,
};

pub const COMPONENT: &str = "component";
pub const STORAGE: &str = "storage";
pub const REQUIRE: &str = "require";

pub const ON_ADD: &str = "on_add";
pub const ON_INSERT: &str = "on_insert";
pub const ON_REPLACE: &str = "on_replace";
pub const ON_REMOVE: &str = "on_remove";
pub const ON_DESPAWN: &str = "on_despawn";

pub const IMMUTABLE: &str = "immutable";

// values for `storage` attribute
const TABLE: &str = "Table";
const SPARSE_SET: &str = "SparseSet";

#[derive(Clone, Copy)]
pub enum StorageTy {
    Table,
    SparseSet,
}

pub struct Attrs {
    pub storage: StorageTy,
    pub requires: Option<Punctuated<Require, Comma>>,
    pub on_add: Option<HookAttributeKind>,
    pub on_insert: Option<HookAttributeKind>,
    pub on_replace: Option<HookAttributeKind>,
    pub on_remove: Option<HookAttributeKind>,
    pub on_despawn: Option<HookAttributeKind>,
    pub relationship: Option<Relationship>,
    pub relationship_target: Option<RelationshipTarget>,
    pub immutable: bool,
}

/// All allowed attribute value expression kinds for component hooks
#[derive(Debug)]
pub enum HookAttributeKind {
    /// expressions like function or struct names
    ///
    /// structs will throw compile errors on the code generation so this is safe
    Path(ExprPath),
    /// function call like expressions
    Call(ExprCall),
}

impl HookAttributeKind {
    pub fn from_expr(value: Expr) -> Result<Self> {
        match value {
            Expr::Path(path) => Ok(HookAttributeKind::Path(path)),
            Expr::Call(call) => Ok(HookAttributeKind::Call(call)),
            // throw meaningful error on all other expressions
            _ => Err(syn::Error::new(
                value.span(),
                [
                    "Not supported in this position, please use one of the following:",
                    "- path to function",
                    "- call to function yielding closure",
                ]
                .join("\n"),
            )),
        }
    }

    pub fn to_token_stream(&self, obel_ecs_path: &Path) -> TokenStream {
        match self {
            HookAttributeKind::Path(path) => path.to_token_stream(),
            HookAttributeKind::Call(call) => {
                quote!({
                    fn _internal_hook(world: #obel_ecs_path::world::DeferredWorld, ctx: #obel_ecs_path::component::HookContext) {
                        (#call)(world, ctx)
                    }
                    _internal_hook
                })
            }
        }
    }
}

impl Parse for HookAttributeKind {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        input.parse::<Expr>().and_then(Self::from_expr)
    }
}

pub fn storage_path(obel_ecs_path: &Path, ty: StorageTy) -> TokenStream {
    let storage_type = match ty {
        StorageTy::Table => Ident::new("Table", Span::call_site()),
        StorageTy::SparseSet => Ident::new("SparseSet", Span::call_site()),
    };

    quote! { #obel_ecs_path::component::StorageType::#storage_type }
}

pub fn hook_register_function_call(
    obel_ecs_path: &Path,
    hook: TokenStream,
    function: Option<TokenStream>,
) -> Option<TokenStream> {
    function.map(|meta| {
        quote! {
            fn #hook() -> ::core::option::Option<#obel_ecs_path::component::ComponentHook> {
                ::core::option::Option::Some(#meta)
            }
        }
    })
}

pub fn parse_component_attr(ast: &DeriveInput) -> Result<Attrs> {
    let mut attrs = Attrs {
        storage: StorageTy::Table,
        on_add: None,
        on_insert: None,
        on_replace: None,
        on_remove: None,
        on_despawn: None,
        requires: None,
        relationship: None,
        relationship_target: None,
        immutable: false,
    };

    let mut require_paths = HashSet::new();
    for attr in ast.attrs.iter() {
        if attr.path().is_ident(COMPONENT) {
            attr.parse_nested_meta(|nested| {
                if nested.path.is_ident(STORAGE) {
                    attrs.storage = match nested.value()?.parse::<LitStr>()?.value() {
                        s if s == TABLE => StorageTy::Table,
                        s if s == SPARSE_SET => StorageTy::SparseSet,
                        s => {
                            return Err(nested.error(format!(
                                "Invalid storage type `{s}`, expected '{TABLE}' or '{SPARSE_SET}'.",
                            )));
                        }
                    };
                    Ok(())
                } else if nested.path.is_ident(ON_ADD) {
                    attrs.on_add = Some(nested.value()?.parse::<HookAttributeKind>()?);
                    Ok(())
                } else if nested.path.is_ident(ON_INSERT) {
                    attrs.on_insert = Some(nested.value()?.parse::<HookAttributeKind>()?);
                    Ok(())
                } else if nested.path.is_ident(ON_REPLACE) {
                    attrs.on_replace = Some(nested.value()?.parse::<HookAttributeKind>()?);
                    Ok(())
                } else if nested.path.is_ident(ON_REMOVE) {
                    attrs.on_remove = Some(nested.value()?.parse::<HookAttributeKind>()?);
                    Ok(())
                } else if nested.path.is_ident(ON_DESPAWN) {
                    attrs.on_despawn = Some(nested.value()?.parse::<HookAttributeKind>()?);
                    Ok(())
                } else if nested.path.is_ident(IMMUTABLE) {
                    attrs.immutable = true;
                    Ok(())
                } else {
                    Err(nested.error("Unsupported attribute"))
                }
            })?;
        } else if attr.path().is_ident(REQUIRE) {
            let punctuated =
                attr.parse_args_with(Punctuated::<Require, Comma>::parse_terminated)?;
            for require in punctuated.iter() {
                if !require_paths.insert(require.path.to_token_stream().to_string()) {
                    return Err(syn::Error::new(
                        require.path.span(),
                        "Duplicate required components are not allowed.",
                    ));
                }
            }
            if let Some(current) = &mut attrs.requires {
                current.extend(punctuated);
            } else {
                attrs.requires = Some(punctuated);
            }
        } else if attr.path().is_ident(RELATIONSHIP) {
            let relationship = attr.parse_args::<Relationship>()?;
            attrs.relationship = Some(relationship);
        } else if attr.path().is_ident(RELATIONSHIP_TARGET) {
            let relationship_target = attr.parse_args::<RelationshipTarget>()?;
            attrs.relationship_target = Some(relationship_target);
        }
    }

    Ok(attrs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::ToString;
    use syn::{DeriveInput, parse_quote};

    #[test]
    fn test_parse_component_attr() {
        let input: DeriveInput = parse_quote! {
            #[component(storage = "Table", on_add = on_add_fn)]
            struct MyComponent;
        };

        let attrs = parse_component_attr(&input).unwrap();
        assert!(matches!(attrs.storage, StorageTy::Table));
        assert!(attrs.on_add.is_some());
        assert!(attrs.on_insert.is_none());
        assert!(attrs.on_replace.is_none());
        assert!(attrs.on_remove.is_none());
        assert!(attrs.on_despawn.is_none());
        assert!(attrs.requires.is_none());
        assert!(attrs.relationship.is_none());
        assert!(attrs.relationship_target.is_none());
        assert!(!attrs.immutable);
    }

    #[test]
    fn test_parse_component_attr_with_requires() {
        let input: DeriveInput = parse_quote! {
            #[component(storage = "Table")]
            #[require(MyRequiredComponent)]
            struct MyComponent;
        };

        let attrs = parse_component_attr(&input).unwrap();
        assert!(matches!(attrs.storage, StorageTy::Table));
        assert!(attrs.requires.is_some());
        let requires = attrs.requires.unwrap();
        assert_eq!(requires.len(), 1);
        assert_eq!(requires[0].path.to_token_stream().to_string(), "MyRequiredComponent");
    }

    #[test]
    fn test_parse_component_attr_with_relationship() {
        let input: DeriveInput = parse_quote! {
            #[component(storage = "Table")]
            #[relationship(relationship_target = MyRelationshipTarget)]
            struct MyComponent;
        };

        let attrs = parse_component_attr(&input).unwrap();
        assert!(matches!(attrs.storage, StorageTy::Table));
        assert!(attrs.relationship.is_some());
        let relationship = attrs.relationship.unwrap();
        assert_eq!(
            relationship.relationship_target.to_token_stream().to_string(),
            "MyRelationshipTarget"
        );
    }

    #[test]
    fn test_parse_component_attr_with_relationship_target() {
        let input: DeriveInput = parse_quote! {
            #[component(storage = "Table")]
            #[relationship_target(relationship = MyRelationship, linked_spawn)]
            struct MyComponent;
        };

        let attrs = parse_component_attr(&input).unwrap();
        assert!(matches!(attrs.storage, StorageTy::Table));
        assert!(attrs.relationship_target.is_some());
        let relationship_target = attrs.relationship_target.unwrap();
        assert_eq!(
            relationship_target.relationship.to_token_stream().to_string(),
            "MyRelationship"
        );
        assert!(relationship_target.linked_spawn);
    }
}
