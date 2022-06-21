use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Write;
use pdb::{FallibleIterator, PdbInternalSectionOffset, RawString, SymbolData};
use reqwest::StatusCode;

pub async fn cache_pdb(name:&str,uuid:&str)->Result<String,Box<dyn Error>>{
    let path = format!(".\\cache\\{}\\{}_{}",name,uuid,name);
    //println!("http://msdl.blackint3.com:88/download/symbols/{}/{}/{}", name, uuid, name);
    if !std::path::Path::new(&path).exists() {
        let response = reqwest::get(&format!("http://msdl.blackint3.com:88/download/symbols/{}/{}/{}", name, uuid, name)).await?;
        let status = response.status();
        if status!=StatusCode::OK{
            return Err(format!("访问符号服务器错误 代码:{}",status).into());
        }

        fs::create_dir_all(&format!(".\\cache\\{}",name))?;
        let mut file = fs::File::create(&path)?;
        let bytes=  response.bytes().await?;
        file.write(bytes.as_ref())?;
    }
    Ok(path)
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
pub fn unwrap_pdb_filter(path:&str,filter:Vec<String>)->Result<Vec<u32>,Box<dyn Error>>{
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