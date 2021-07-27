use async_fs::{DirEntry, File};
use async_walkdir::WalkDir;
use futures_lite::io::AsyncReadExt;
use futures_lite::stream::StreamExt;
use blake2::{Blake2b, Digest};
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
    let blake2b = hash.finalize()[0..32].try_into().unwrap();
    Ok(Some(MediaFile {
        file_path,
        blake2b,
        bytes,
    }))
}

pub const PATHS: &[u8; 1] = b"p";
pub const HASHES: &[u8; 1] = b"h";

pub struct Metadb {
    feed: hypercore::Feed<random_access_disk::RandomAccessDisk>,
    pub paths_to_hashes: sled::Tree,
    pub hashes_to_paths: sled::Tree,
}

impl Metadb
{
    pub async fn new(
        storage: impl AsRef<Path>,
    ) -> Result<Self, std::io::Error> {

        let mut hypercore_dir = storage.as_ref().to_owned();
        hypercore_dir.push("feed");
        // TODO propagate errors
        let feed = hypercore::open(hypercore_dir).await.unwrap();

        let mut db_dir = storage.as_ref().to_owned();
        db_dir.push("db");
        let db = sled::open(db_dir).expect("open");
        let paths_to_hashes = db.open_tree(PATHS).unwrap();
        let hashes_to_paths = db.open_tree(HASHES).unwrap();

        Ok(Metadb {
            feed,
            paths_to_hashes,
            hashes_to_paths,
        })
    }

    /// Scan a directory and return the number of entries processed
    pub async fn scan(
        &mut self,
        root: impl AsRef<Path>,
    ) -> Result<u32, std::io::Error> {

        let mut added_entries = 0;
        let mut entries = WalkDir::new(root);
        loop {
            match entries.next().await {
                Some(Ok(entry)) => match process_entry(entry).await {
                    Ok(Some(media_file)) => {
                        println!("{:x?}", media_file);
                        match media_file.file_path.clone().into_os_string().into_string() {
                            Ok(s) => {
                                let encoded = bincode::serialize(&media_file).unwrap();
                                self.feed.append(&encoded).await.unwrap();
                                // let seq = self.feed.len();
                                added_entries += 1;
                                self.hashes_to_paths
                                    .insert(media_file.blake2b, s.as_str())
                                    .unwrap();
                                self.paths_to_hashes
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
        Ok(added_entries)
    }

    pub fn num_files(&self) -> u64 {
       self.feed.len()
    }
}
