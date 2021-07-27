#[cfg(test)]
use futures_lite::future;
use crate::scan::scan;
use crate::scan::PATHS;
use crate::scan::HASHES;
use tempfile::TempDir;

#[test]
fn it_works() {
    let db = sled::open("./db").expect("open");
    let storage = TempDir::new().unwrap();
    let scan_result: Result<u32, std::io::Error> =
        future::block_on(async { scan("./test-media", &db, storage).await });
    assert_eq!(scan_result.unwrap(), 1);
    let paths_to_hashes = db.open_tree(PATHS).unwrap();
    let hashes_to_paths = db.open_tree(HASHES).unwrap();
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
