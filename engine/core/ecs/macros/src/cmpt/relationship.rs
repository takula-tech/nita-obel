use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::format;
use syn::{
    Data, DataStruct, DeriveInput, Field, Fields, Member, Path, Result, Token, Type, Visibility,
    parse::Parse, spanned::Spanned,
};

use super::Attrs;

pub const RELATIONSHIP: &str = "relationship";
pub const RELATIONSHIP_TARGET: &str = "relationship_target";

mod kw {
    syn::custom_keyword!(relationship_target);
    syn::custom_keyword!(relationship);
    syn::custom_keyword!(linked_spawn);
}

pub struct Relationship {
    pub relationship_target: Type,
}

pub struct RelationshipTarget {
    pub relationship: Type,
    pub linked_spawn: bool,
}

impl Parse for Relationship {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        input.parse::<kw::relationship_target>()?;
        input.parse::<Token![=]>()?;
        Ok(Relationship {
            relationship_target: input.parse::<Type>()?,
        })
    }
}

impl Parse for RelationshipTarget {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut relationship: Option<Type> = None;
        let mut linked_spawn: bool = false;

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::linked_spawn) {
                input.parse::<kw::linked_spawn>()?;
                linked_spawn = true;
            } else if lookahead.peek(kw::relationship) {
                input.parse::<kw::relationship>()?;
                input.parse::<Token![=]>()?;
                relationship = Some(input.parse()?);
            } else {
                return Err(lookahead.error());
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(RelationshipTarget {
            relationship: relationship.ok_or_else(|| {
                syn::Error::new(input.span(), "Missing `relationship = X` attribute")
            })?,
            linked_spawn,
        })
    }
}

pub fn derive_relationship(
    ast: &DeriveInput,
    attrs: &Attrs,
    obel_ecs_path: &Path,
) -> Result<Option<TokenStream>> {
    let Some(relationship) = &attrs.relationship else {
        return Ok(None);
    };
    let Data::Struct(DataStruct {
        fields,
        struct_token,
        ..
    }) = &ast.data
    else {
        return Err(syn::Error::new(ast.span(), "Relationship can only be derived for structs."));
    };
    let field = relationship_field(fields, "Relationship", struct_token.span())?;

    let relationship_member = field.ident.clone().map_or(Member::from(0), Member::Named);
    let members = fields.members().filter(|member| member != &relationship_member);

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    let relationship_target = &relationship.relationship_target;

    Ok(Some(quote! {
        impl #impl_generics #obel_ecs_path::relationship::Relationship for #struct_name #type_generics #where_clause {
            type RelationshipTarget = #relationship_target;

            #[inline(always)]
            fn get(&self) -> #obel_ecs_path::entity::Entity {
                self.#relationship_member
            }

            #[inline]
            fn from(entity: #obel_ecs_path::entity::Entity) -> Self {
                Self {
                    #(#members: core::default::Default::default(),),*
                    #relationship_member: entity
                }
            }
        }
    }))
}

pub fn derive_relationship_target(
    ast: &DeriveInput,
    attrs: &Attrs,
    obel_ecs_path: &Path,
) -> Result<Option<TokenStream>> {
    let Some(relationship_target) = &attrs.relationship_target else {
        return Ok(None);
    };

    let Data::Struct(DataStruct {
        fields,
        struct_token,
        ..
    }) = &ast.data
    else {
        return Err(syn::Error::new(
            ast.span(),
            "RelationshipTarget can only be derived for structs.",
        ));
    };
    let field = relationship_field(fields, "RelationshipTarget", struct_token.span())?;

    if field.vis != Visibility::Inherited {
        return Err(syn::Error::new(
            field.span(),
            "The collection in RelationshipTarget must be private to prevent users from directly mutating it, which could invalidate the correctness of relationships.",
        ));
    }
    let collection = &field.ty;
    let relationship_member = field.ident.clone().map_or(Member::from(0), Member::Named);

    let members = fields.members().filter(|member| member != &relationship_member);

    let relationship = &relationship_target.relationship;
    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();
    let linked_spawn = relationship_target.linked_spawn;
    Ok(Some(quote! {
        impl #impl_generics #obel_ecs_path::relationship::RelationshipTarget for #struct_name #type_generics #where_clause {
            const LINKED_SPAWN: bool = #linked_spawn;
            type Relationship = #relationship;
            type Collection = #collection;

            #[inline]
            fn collection(&self) -> &Self::Collection {
                &self.#relationship_member
            }

            #[inline]
            fn collection_mut_risky(&mut self) -> &mut Self::Collection {
                &mut self.#relationship_member
            }

            #[inline]
            fn from_collection_risky(collection: Self::Collection) -> Self {
                Self {
                    #(#members: core::default::Default::default(),),*
                    #relationship_member: collection
                }
            }
        }
    }))
}

/// Returns the field with the `#[relationship]` attribute, the only field if unnamed,
/// or the only field in a [`Fields::Named`] with one field, otherwise `Err`.
pub fn relationship_field<'a>(
    fields: &'a Fields,
    derive: &'static str,
    span: Span,
) -> Result<&'a Field> {
    match fields {
      Fields::Named(fields) if fields.named.len() == 1 => Ok(fields.named.first().unwrap()),
      Fields::Named(fields) => fields.named.iter().find(|field| {
          field
              .attrs
              .iter()
              .any(|attr| attr.path().is_ident(RELATIONSHIP))
      }).ok_or(syn::Error::new(
          span,
          format!("{derive} derive expected named structs with a single field or with a field annotated with #[relationship].")
      )),
      Fields::Unnamed(fields) => fields
          .unnamed
          .len()
          .eq(&1)
          .then(|| fields.unnamed.first())
          .flatten()
          .ok_or(syn::Error::new(
              span,
              format!("{derive} derive expected unnamed structs with one field."),
          )),
      Fields::Unit => Err(syn::Error::new(
          span,
          format!("{derive} derive expected named or unnamed struct, found unit struct."),
      )),
  }
}
