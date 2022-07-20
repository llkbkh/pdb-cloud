mod qs;
mod lib;
use serde::{Serialize, Deserialize};
use std::error::Error;
use std::mem;
use std::net::SocketAddr;
use tokio;
use qs::QsSerde;
use axum::{routing::get, Router, extract::{
    Path
}, Json};
use base64;
use tracing;
use bincode;

#[derive(serde::Deserialize, Debug)]
struct Input {
    #[serde(skip_serializing_if = "Option::is_none")]
    names: Option<Vec<String>>,
}
async fn query_symbol(Path((uuid,name)):Path<(String, String)>, QsSerde(m):QsSerde<Input>) ->String{
    return match lib::cache_pdb(&name, &uuid).await {
        Ok(path) => {
            match m.names {
                Some(names) => {
                    match lib::unwrap_pdb_filter(&path, &names) {
                        Ok(mut v) => {
                            v.reverse();
                            format!("{:?}", v)
                        },
                        Err(e) => {
                            tracing::error!("解包符号错误:{}", e);
                            format!("{}", e)
                        }
                    }
                },
                None => {
                    match lib::unwrap_pdb(&path) {
                        Ok(map) => format!("{:?}", map),
                        Err(e) => {
                            tracing::error!("解包符号错误:{}", e);
                            format!("{:?}", e)
                        },
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("缓存符号错误:{}", e);
            format!("{}", e)
        }
    }
}

#[derive(Serialize, Debug)]
#[repr(C)]
struct DriverRely{
    pub MiGetPage:u32,
    pub MiInitializePfn:u32,
    pub MiSystemPartition:u32,
    pub MiZeroPhysicalPage:u32,
    pub PspCidTable:u32,
    pub ActiveProcessLinks:u64,
}
impl DriverRely{
    pub fn get(path:&str)->Self{
        let l: Vec<String> = vec![
            "MiGetPage".to_string(),
            "MiInitializePfn".to_string(),
            "MiSystemPartition".to_string(),
            "MiZeroPhysicalPage".to_string(),
            "PspCidTable".to_string(),
        ];
        tracing::info!("DriverRely get {}", path);
        let ActiveProcessLinks = {
            let mut result = 0;
            if let Ok(data) = lib::unwrap_pdb_class_filter(&path, &vec!["_EPROCESS".to_string()]){
                result = data[&"_EPROCESS".to_string()][&"ActiveProcessLinks".to_string()]
            }
            result
        };
        return match lib::unwrap_pdb_filter_map(path,&l) {
            Ok(data) => {
                tracing::info!("解析成功");
                Self{
                    MiGetPage: match data.get("MiGetPage") {
                        Some(d) => *d,
                        None => 0
                    },
                    MiInitializePfn: match data.get("MiInitializePfn") {
                        Some(d) => *d,
                        None => 0
                    },
                    MiSystemPartition: match data.get("MiSystemPartition") {
                        Some(d) => *d,
                        None => 0
                    },
                    MiZeroPhysicalPage: match data.get("MiZeroPhysicalPage") {
                        Some(d) => *d,
                        None => 0
                    },
                    PspCidTable: match data.get("PspCidTable") {
                        Some(d) => *d,
                        None => 0
                    },
                    ActiveProcessLinks,
                }
            }
            Err(_) =>{
                tracing::error!("解析失败");
                Self{
                MiGetPage: 0,
                MiInitializePfn: 0,
                MiSystemPartition: 0,
                MiZeroPhysicalPage: 0, 
                    PspCidTable: 0, 
                    ActiveProcessLinks:0,
                
            }}
        };

    }
}

async fn driver(Path((name,uuid)):Path<(String, String)>)->String{
    tracing::info!("cache_pdb->name :{},uuid:{}",name,uuid);
    let s =  match lib::cache_pdb(&name, &uuid).await{
        Ok(path) => DriverRely::get(&path),
        Err(e) => {
            tracing::error!("缓存符号错误 :{}",e);
            DriverRely{
                MiGetPage: 0,
                MiInitializePfn: 0,
                MiSystemPartition: 0,
                MiZeroPhysicalPage: 0,
                PspCidTable: 0,
                ActiveProcessLinks: 0
            }
        }
    };
    tracing::info!("bincode");
    let sb:Vec<u8> = match bincode::serialize(&s) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("序列化错误 :{}",e);
            Vec::from([0;std::mem::size_of::<DriverRely>()])
        }
    };
    base64::encode(sb)
}


#[tokio::main]
async fn main(){
    // if std::env::var_os("RUST_LOG").is_none() {
    //     std::env::set_var("RUST_LOG", "example_all=debug,tower_http=debug")
    // }



    tracing_subscriber::fmt::init();
    let app = Router::new()
        .route("/",get( ||async {"hello world"}))
        .route("/query/:uuid/:name",get(query_symbol))
        .route("/driver/:name/:uuid",get(driver));
    let addr:SocketAddr = "127.0.0.1:3000".parse().unwrap();
    tracing::info!("{}",&addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await.unwrap();
}
