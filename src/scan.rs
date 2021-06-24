use async_fs::{DirEntry, File};
use async_walkdir::WalkDir;
use futures_lite::io::AsyncReadExt;
use futures_lite::stream::StreamExt;
use blake2::{Blake2b, Digest};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::convert::TryInto;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct MediaFile {
    file_path: PathBuf,
    blake2b: [u8; 32],
    bytes: u64,
}

pub async fn process_entry(entry: DirEntry) -> Result<Option<MediaFile>, std::io::Error> {
    const BLOCK_SIZE: usize = 64 * 1024;
    if entry.file_type().await?.is_dir() {
        return Ok(None);
    }
    let mut file = File::open(entry.path()).await?;
    let file_path = entry.path();
    let bytes = file.metadata().await?.len();

    let mut buf = vec![0; BLOCK_SIZE];
    let mut hash = Blake2b::new();
    loop {
        let bytes_read = file.read(&mut buf).await?;
        if bytes_read == 0 {
            break;
        }
        hash.update(&buf[0..bytes_read]);
    }
    let blake2b = hash.finalize().as_slice().try_into().unwrap();
    Ok(Some(MediaFile {
        file_path,
        blake2b,
        bytes,
    }))
}

pub async fn scan(
    root: impl AsRef<Path>,
    db: &sled::Db,
) -> Result<HashMap<[u8; 32], PathBuf>, std::io::Error> {
    let mut entries = WalkDir::new(root);
    let media_files = HashMap::new();
    let paths_to_hashes = db.open_tree(b"p").unwrap();
    let hashes_to_paths = db.open_tree(b"h").unwrap();
    let mut feed = hypercore::open("./feed.db").unwrap();
    loop {
        match entries.next().await {
            Some(Ok(entry)) => match process_entry(entry).await {
                Ok(Some(media_file)) => {
                    println!("{:x?}", media_file);
                    // media_files.insert(media_file.sha256, &media_file.file_path);
                    match media_file.file_path.clone().into_os_string().into_string() {
                        Ok(s) => {
                            let encoded = bincode::serialize(&media_file).unwrap();
                            feed.append(encoded);
                            hashes_to_paths
                                .insert(media_file.blake2b, s.as_str())
                                .unwrap();
                            paths_to_hashes
                                .insert(s.as_bytes(), media_file.blake2b.to_vec())
                                .unwrap();
                        }
                        Err(_e) => {}
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    eprint!("Errord {}", e);
                    break;
                }
            },
            Some(Err(e)) => {
                eprintln!("Error {}", e);
                break;
            }
            None => break,
        };
    }
    Ok(media_files)
}
