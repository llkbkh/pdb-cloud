use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Write;
use pdb::{FallibleIterator};
use reqwest::StatusCode;
use tracing;
pub async fn cache_pdb(name:&str,uuid:&str)->Result<String,Box<dyn Error>>{
    let path = format!("cache/{}/{}_{}",name,uuid,name);
    tracing::info!("cache_pdb->path :{}",path);
    if !std::path::Path::new(&path).exists() {
        let req_path = format!("https://msdl.microsoft.com/download/symbols/{}/{}/{}", name, uuid, name);
        tracing::info!("reqwest->get:{}",req_path);
        let response = reqwest::get(&req_path).await?;
        let status = response.status();
        tracing::info!("reqwest->status:{}",status);
        if status!=StatusCode::OK{
            return Err(format!("访问符号服务器错误 代码:{}",status).into());
        }
        fs::create_dir_all(&format!("cache/{}",name))?;
        let mut file = fs::File::create(&path)?;
        let bytes=  response.bytes().await?;
        file.write(bytes.as_ref())?;
    }
    Ok(path)
}
pub fn unwrap_pdb_class(path:&str)->Result<HashMap<String,HashMap<String,u64>>,Box<dyn Error>>{
    let mut result= HashMap::new();
    let file = fs::File::open(path)?;

    let mut pdb = pdb::PDB::open(&file)?;

    let type_information = pdb.type_information()?;
    let mut type_finder = type_information.finder();
    let mut type_iter = type_information.iter();
    while let Some(typ) = type_iter.next()? {
        // keep building the index
        type_finder.update(&type_iter);

        if let Ok(pdb::TypeData::Class(class)) = typ.parse() {
            if !class.properties.forward_reference()
            {
                let mut map = HashMap::new();

                if let Some(fields) = class.fields {
                    if let pdb::TypeData::FieldList(data) = type_finder.find(fields)?.parse()? {
                        for field in data.fields {
                            if let pdb::TypeData::Member(member) = field {
                                map.insert(member.name.to_string().to_string(),member.offset);
                            }
                        }
                    }
                }
                if !map.is_empty(){
                    result.insert(class.name.to_string().to_string(),map);
                }

            }
        }
    }
    Ok(result)
}
pub fn unwrap_pdb_class_filter(path:&str,filter:&Vec<String>)->Result<HashMap<String,HashMap<String,u64>>,Box<dyn Error>>{
    let mut result= HashMap::new();
    let file = fs::File::open(path)?;

    let mut pdb = pdb::PDB::open(&file)?;

    let type_information = pdb.type_information()?;
    let mut type_finder = type_information.finder();
    let mut type_iter = type_information.iter();
    while let Some(typ) = type_iter.next()? {
        // keep building the index
        type_finder.update(&type_iter);

        if let Ok(pdb::TypeData::Class(class)) = typ.parse() {
            if filter.contains(&class.name.to_string().to_string()) &&
                !class.properties.forward_reference()
            {
                let mut map = HashMap::new();
                if let Some(fields) = class.fields {
                    if let pdb::TypeData::FieldList(data) = type_finder.find(fields)?.parse()? {
                        for field in data.fields {
                            if let pdb::TypeData::Member(member) = field {
                                map.insert(member.name.to_string().to_string(),member.offset);
                            }
                        }
                    }
                }
                if !map.is_empty(){
                    result.insert(class.name.to_string().to_string(),map);
                }

            }
        }
    }
    Ok(result)
}
pub fn unwrap_pdb(path:&str)->Result<HashMap<String,u32>,Box<dyn Error>>{
    let mut map = HashMap::new();
    let file = fs::File::open(&path)?;
    let mut pdb = pdb::PDB::open(file)?;

    let symbol_table = pdb.global_symbols()?;
    let address_map = pdb.address_map()?;

    let mut symbols = symbol_table.iter();
    while let Some(symbol) = symbols.next()? {
        match symbol.parse() {
            Ok(pdb::SymbolData::Public(data)) => {
                // we found the location of a function!
                let rva = data.offset.to_rva(&address_map).unwrap_or_default();
                map.insert( data.name.to_string().to_string(),rva.0);
            }
            _ => {}
        }
    }
    Ok(map)
}
pub fn unwrap_pdb_filter(path:&str, filter: &Vec<String>) ->Result<Vec<u32>,Box<dyn Error>>{
    let mut vec:Vec<u32> = Vec::new();
    let file = fs::File::open(&path)?;
    let mut pdb = pdb::PDB::open(file)?;

    let symbol_table = pdb.global_symbols()?;
    let address_map = pdb.address_map()?;

    let mut symbols = symbol_table.iter();
    while let Some(symbol) = symbols.next()? {
        match symbol.parse() {
            Ok(pdb::SymbolData::Public(data)) => {
                // we found the location of a function!
                let rva = data.offset.to_rva(&address_map).unwrap_or_default();
                if filter.contains(&data.name.to_string().to_string()){
                    vec.push(rva.0);
                }
            }
            _ => {}
        }
    }
    Ok(vec)
}
pub fn unwrap_pdb_filter_map(path:&str, filter: &Vec<String>) ->Result<HashMap<String,u32>,Box<dyn Error>>{
    let mut map:HashMap<String,u32> = HashMap::new();
    let file = fs::File::open(&path)?;
    let mut pdb = pdb::PDB::open(file)?;

    let symbol_table = pdb.global_symbols()?;
    let address_map = pdb.address_map()?;

    let mut symbols = symbol_table.iter();
    while let Some(symbol) = symbols.next()? {
        match symbol.parse() {
            Ok(pdb::SymbolData::Public(data)) => {
                // we found the location of a function!
                let rva = data.offset.to_rva(&address_map).unwrap_or_default();
                if filter.contains(&data.name.to_string().to_string()){
                    map.insert(data.name.to_string().to_string(),rva.0);
                }
            }
            _ => {}
        }
    }
    Ok(map)
}
