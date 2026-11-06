use std::{fmt, io, marker::PhantomData};

use bevy::{
    asset::{Asset, AssetLoader, LoadContext, io::Reader},
    reflect::Reflect,
};
use serde::Deserialize;

#[derive(Reflect)]
pub struct JsonLoader<A>(PhantomData<A>);

impl<A> Default for JsonLoader<A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[derive(Debug)]
pub enum JsonLoaderError {
    JsonError(serde_json::error::Error),
    IoError(io::Error),
}

impl core::fmt::Display for JsonLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JsonError(error) => error.fmt(f),
            Self::IoError(error) => error.fmt(f),
        }
    }
}

impl core::error::Error for JsonLoaderError {}

impl<A: Asset + for<'de> Deserialize<'de> + 'static> AssetLoader for JsonLoader<A> {
    type Asset = A;

    type Settings = ();

    type Error = JsonLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &(),
        _: &mut LoadContext<'_>,
    ) -> Result<A, JsonLoaderError> {
        let mut buf = Vec::new();
        reader
            .read_to_end(&mut buf)
            .await
            .map_err(|e| JsonLoaderError::IoError(e))?;

        let st = String::from_utf8(buf).map_err(|_| {
            JsonLoaderError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not UTF-8",
            ))
        })?;

        let asset = serde_json::from_str(&st).map_err(JsonLoaderError::JsonError)?;

        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}
