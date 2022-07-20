use std::convert::Infallible;
use axum::extract::{FromRequest, RequestParts};

pub struct QsSerde<T>(pub T);

#[axum::async_trait]
impl<B, T> FromRequest<B> for QsSerde<T>
    where
        T: serde::de::DeserializeOwned,
        B: Send
{
    type Rejection = Infallible;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        match req.uri().query() {
            Some(params) => {
                match serde_qs::from_str(params){
                    Ok(m)=>Ok(Self(m)),
                    Err(e)=>{
                        tracing::error!("错误参数 :{:?}",e);
                        Ok(Self(serde_qs::from_str("").unwrap()))
                    }
                }
                //Ok(Self(serde_qs::from_str(params).unwrap()))
            }
            None => {
                Ok(Self(serde_qs::from_str("").unwrap()))
            }
        }
    }
}