use std::env::var_os;
use std::path::{Path,PathBuf};

use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;

use std::collections::HashMap;
use std::ptr::null_mut;
use winapi::shared::minwindef::*;
use winapi::um::wincrypt::DATA_BLOB;
use winapi::um::dpapi::*;
use base64;

fn decrypt(b64: &str) -> String {

    let data = &base64::decode(b64).unwrap()[..];
    unsafe{
        let mut indata = DATA_BLOB {
            cbData: data.len() as DWORD,
            pbData: data.as_ptr() as *mut BYTE,
        };
        let mut outdata = DATA_BLOB{
            cbData: 0,
            pbData: 0 as *mut _,
        };
        
        CryptUnprotectData(&mut indata,null_mut(),null_mut(),null_mut(),null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,&mut outdata);
        let len = outdata.cbData as usize;
        String::from_raw_parts(outdata.pbData, len, len)
    }
}

fn read_svn(name: &Path) {
    let f = File::open(name).unwrap();
    let f = BufReader::new(f);
    let mut lines_iter = f.lines().map(|l| l.unwrap());
    let mut config = HashMap::new();

    loop {
        let mut key = String::new();
        let mut value = String::new();

        if let Some(line) = lines_iter.next(){
            if line.starts_with("K") {
                if let Some(line) = lines_iter.next(){
                    key.push_str(line.as_str());
                }
            }else if line.starts_with("END") {
                break;
            }
        }else{
            break;
        }

        if let Some(line) = lines_iter.next(){

            if line.starts_with("V") {
                if let Some(line) = lines_iter.next(){
                    value.push_str(line.as_str());
                }
            }else if line.starts_with("END") {
                break;
            }
        }else {
            break;
        }

        if ! key.is_empty() && ! value.is_empty() {
            config.insert(key, value);
        }

    }

    for (key, value) in &config {
        if key == "password" {
            println!("{}: {}", key, decrypt(value));
        }else{
            println!("{}: {}", key, value);
        }
    }
    println!();

}

fn parse_svn(){
    let svn_path = var_os("AppData")
        .map(PathBuf::from)
        .map(|d| d.join("Subversion\\auth\\svn.simple")).unwrap();
    assert_eq!(svn_path.exists(), true);
    for entry in svn_path.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            // println!("{:?}", entry.path());
            read_svn(&entry.path());
        }
    }
}

fn main(){
    parse_svn();
}
