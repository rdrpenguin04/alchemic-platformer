use std::marker::PhantomData;

use bevy::{asset::{Asset, AssetLoader}, ecs::error::BevyError, reflect::Reflect};


#[derive(Reflect)]
pub struct JsonLoader<A> (PhantomData<A>);


#[derive(Debug)]
pub enum JsonLoaderError{
    JsonError(serde_json::error::Error),
    IoError(std::io::Error)
}

impl core::fmt::Display for JsonLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonLoaderError::JsonError(error) => error.fmt(f),
            JsonLoaderError::IoError(error) => error.fmt(f),
        }
    }
}

impl core::error::Error for JsonLoaderError {}


impl<A: Asset + for<'de> serde::de::Deserialize<'de> + 'static> AssetLoader for JsonLoader<A> {
    type Asset = A;

    type Settings = ();

    type Error = JsonLoaderError;

    fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _: &Self::Settings,
        _: &mut bevy::asset::LoadContext,
    ) -> impl bevy::tasks::ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).await.map_err(|e| JsonLoaderError::IoError(e))?;

            let st = String::from_utf8(buf)
                .map_err(|_| JsonLoaderError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, "Not UTF-8")))?;

            let asset = serde_json::from_str(&st)
                .map_err(JsonLoaderError::JsonError)?;

            Ok(asset)
        }
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}