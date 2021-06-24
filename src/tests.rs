#[cfg(test)]
use futures_lite::future;
use crate::scan::scan;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn it_works() {
    let db = sled::open("./db").expect("open");
    let scan_result: Result<HashMap<[u8; 32], PathBuf>, std::io::Error> =
        future::block_on(async { scan("./test-media", &db).await });
    assert_eq!(scan_result.unwrap().len(), 0);
    let paths_to_hashes = db.open_tree(b"p").unwrap();
    let hashes_to_paths = db.open_tree(b"h").unwrap();
    for kv in paths_to_hashes.iter() {
        let (key, value) = kv.unwrap();
        println!(
            "key: {:?} value: {:?}",
            std::str::from_utf8(&key).unwrap(),
            value
        );
    }
    for kv in hashes_to_paths.iter() {
        let (key, value) = kv.unwrap();
        println!(
            "key: {:?} value: {:?}",
            key,
            std::str::from_utf8(&value).unwrap()
        );
    }
}
