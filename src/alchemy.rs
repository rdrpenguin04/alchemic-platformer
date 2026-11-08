use std::num::NonZero;

use bevy::{platform::collections::HashMap, prelude::*};
use serde::Deserializer;
use serde_derive::Deserialize;

use crate::helpers::JsonLoader;

pub fn plugin(app: &mut App) {
    app.init_asset_loader::<JsonLoader<Recipe>>()
        .insert_resource(Elements {
            by_id: HashMap::from([
                ("fire".into(), Element::new("fire", "Fire", 4)),
                ("air".into(), Element::new("air", "Air", 8)),
                ("water".into(), Element::new("water", "Water", 12)),
                ("earth".into(), Element::new("earth", "Earth", 16)),
            ]),
        });
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Reflect, Asset)]
pub struct RecipeStack {
    pub id: String,
    pub quantity: u32,
}

impl<'de> serde::de::Deserialize<'de> for RecipeStack {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn validate_id<E: serde::de::Error>(s: &str) -> Result<(), E> {
            let mut chars = s.chars();

            let Some(first) = chars.next() else {
                return Err(E::invalid_value(
                    serde::de::Unexpected::Str(s),
                    &"Empty identifier is not allowed",
                ));
            };

            if !first.is_xid_start() && first != '_' {
                return Err(E::invalid_value(
                    serde::de::Unexpected::Str(s),
                    &"Id must be a legal identifier",
                ));
            }

            for c in chars {
                if !c.is_xid_continue() && c != '_' {
                    return Err(E::invalid_value(
                        serde::de::Unexpected::Str(s),
                        &"Id must be a legal identifier",
                    ));
                }
            }

            Ok(())
        }

        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = RecipeStack;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("An id or an id and quantity object")
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                validate_id(&v)?;
                Ok(RecipeStack { id: v, quantity: 1 })
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                validate_id(v)?;
                Ok(RecipeStack {
                    id: v.to_string(),
                    quantity: 1,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut id = None::<String>;
                let mut quantity = None::<u32>;

                while let Some(k) = map.next_key::<&str>()? {
                    match k {
                        "id" => {
                            if id.is_some() {
                                return Err(<A::Error as serde::de::Error>::duplicate_field("id"));
                            }

                            id = Some(map.next_value()?);
                        }
                        "quantity" => {
                            if quantity.is_some() {
                                return Err(<A::Error as serde::de::Error>::duplicate_field(
                                    "qyantity",
                                ));
                            }

                            quantity = Some(map.next_value()?);
                        }
                        _ => {}
                    }
                }

                let id = id.ok_or_else(|| <A::Error as serde::de::Error>::missing_field("id"))?;
                let quantity = quantity.unwrap_or(1);

                validate_id(&id)?;

                Ok(RecipeStack { id, quantity })
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let id = seq
                    .next_element::<String>()?
                    .ok_or_else(|| <A::Error as serde::de::Error>::missing_field("id"))?;
                let quantity = seq.next_element::<u32>()?.unwrap_or(1);

                validate_id(&id)?;

                Ok(RecipeStack { id, quantity })
            }
        }

        deserializer.deserialize_struct("RecipeStack", &["id", "quantity"], Visitor)
    }
}

#[derive(Deserialize, Clone, Debug, Asset, Reflect)]
pub struct Recipe {
    pub output: RecipeStack,
    #[serde(deserialize_with = "sort_inputs")]
    pub inputs: Vec<RecipeStack>,
}

impl Recipe {
    /// If `inputs` allows this recipe to be crafted, returns the maximum number of outputs of the recipe that can be made, else returns [`None`].
    pub fn can_craft(&self, inputs: &[RecipeStack]) -> Option<NonZero<u32>> {
        if self.inputs.len() != inputs.len() {
            return None;
        }

        let mut max_multiplicity = u32::MAX;

        for (req, input) in self.inputs.iter().zip(inputs) {
            if req.id != input.id {
                return None;
            }

            if req.quantity > input.quantity {
                return None;
            }

            let multiplicity = input.quantity / req.quantity;

            max_multiplicity = max_multiplicity.min(multiplicity);
        }

        Some(NonZero::new(max_multiplicity).expect("Should be at least 1"))
    }
}

fn sort_inputs<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<RecipeStack>, D::Error> {
    use serde::de::Deserialize as _;
    let mut val = Vec::<RecipeStack>::deserialize(de)?;

    val.sort_by(|a, b| Ord::cmp(&a.id, &b.id));

    Ok(val)
}

#[derive(Clone, Reflect)]
pub struct Element {
    pub id: String,
    pub name: String,
    pub icon_id: usize,
}

impl Element {
    pub fn new(id: impl Into<String>, name: impl Into<String>, icon_id: usize) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon_id,
        }
    }
}

#[derive(Resource, Clone, Reflect)]
pub struct Elements {
    pub by_id: HashMap<String, Element>,
}
