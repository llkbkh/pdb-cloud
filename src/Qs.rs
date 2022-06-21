use std::convert::Infallible;
use axum::extract::{FromRequest, RequestParts};

pub struct Qs_Serde<T>(pub T);

#[axum::async_trait]
impl<B, T> FromRequest<B> for Qs_Serde<T>
    where
        T: serde::de::DeserializeOwned,
        B: std::marker::Send
{
    type Rejection = Infallible;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        match req.uri().query() {
            Some(params) => {
                //println!("{}",params);
                Ok(Self(serde_qs::from_str(params).unwrap()))
            }
            None => {
                Ok(Self(serde_qs::from_str("").unwrap()))
            }
        }
    }
}