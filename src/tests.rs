#[cfg(test)]
use futures_lite::future;
use crate::metadb;
use tempfile::TempDir;

#[test]
fn basic() {
    let storage = TempDir::new().unwrap();
    future::block_on(async {
            let mut m = metadb::Metadb::new(storage).await.unwrap();
            assert_eq!(m.scan("./test-media").await.unwrap(), 1);
            assert_eq!(m.num_files(), 1);
            for kv in m.paths_to_hashes.iter() {
                let (key, value) = kv.unwrap();
                println!(
                    "key: {:?} value: {:?}",
                    std::str::from_utf8(&key).unwrap(),
                    value
                );
            }
            for kv in m.hashes_to_paths.iter() {
                let (key, value) = kv.unwrap();
                println!(
                    "key: {:?} value: {:?}",
                    key,
                    std::str::from_utf8(&value).unwrap()
                );
            }
    });

}
