mod Qs;
mod lib;

use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;
use std::fs;
use std::hash::Hash;
use std::io::Write;
use pdb;
use pdb::FallibleIterator;
use reqwest;
use tokio;
use Qs::Qs_Serde;
use axum::{
    routing::get,
    Router,
    extract::{
        Path,
        Query
    },
    Json
};

#[derive(serde::Deserialize, Debug)]
struct Input {
    #[serde(skip_serializing_if = "Option::is_none")]
    names: Option<Vec<String>>,
}

async fn query_symbol(Path((uuid,name)):Path<(String, String)>, Qs_Serde(m):Qs_Serde<Input>) ->String{

    match lib::cache_pdb(&name,&uuid).await {
        Ok(path) => {
            return match m.names {
                Some(names) => {
                    match lib::unwrap_pdb_filter(&path, names) {
                        Ok(mut v) => {
                            v.reverse();
                            format!("{:?}",v)
                        },
                        Err(e) => format!("{}", e)

                    }
                },
                None => {
                    match lib::unwrap_pdb(&path) {
                        Ok(map) => format!("{:?}", map),
                        Err(e) => format!("{:?}", e),
                    }
                }
            }
        }
        Err(e) => {
            return format!("{}", e);
        }
    }
}
#[tokio::main]
async fn main(){
    let app = Router::new()
        .route("/",get( ||async {"hello world"}))
        .route("/test/:uuid/:name",get(query_symbol))
       ;
    axum::Server::bind(&"[::]:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await.unwrap();
}
