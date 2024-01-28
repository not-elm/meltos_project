use std::path::Path;

use async_trait::async_trait;
use meltos_tvc::file_system::{FileSystem, Stat, StatType};
use meltos_util::console_log;
use meltos_util::path::AsUri;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::js_sys::{Object, Uint8Array};

use crate::directory::home_dir;
use crate::file_system::node::fs::{exists_sync, lstat_sync, mkdir_sync, read_dir_sync, read_file_sync, rm_recursive, write_file_sync};

mod buffer;
mod error;
mod stats;
mod fs;

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NodeFileSystem {
    pub workspace_folder: String,
}

#[wasm_bindgen]
impl NodeFileSystem {
    #[wasm_bindgen(constructor)]
    pub fn new(workspace_folder: String) -> Self {
        Self {
            workspace_folder,
        }
    }

    #[inline]
    fn path(&self, path: &str) -> String {
        if path == "./" || path == "." {
            self.workspace_folder.clone()
        } else if path.starts_with(&self.workspace_folder) {
            path.to_string()
        } else {
            format!("{}/{}", self.workspace_folder, path.replace("", ""))
        }
    }
}


impl Default for NodeFileSystem {
    fn default() -> Self {
        let dir = format!("{}/meltos", home_dir());
        if !exists_sync(&dir).unwrap() {
            mkdir_sync(&dir, MkdirOptions {
                recursive: true
            }).unwrap();
        }
        console_log!("home dir = {dir}");
        Self {
            workspace_folder: dir
        }
    }
}

impl NodeFileSystem {
    pub fn write_sync(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        let path = self.path(path);
        if let Some(dir) = Path::new(&path).parent() {
            let dir = dir.as_uri();
            self.create_dir_sync(&dir)?;
        }
        write_file_sync(&path, buf.to_vec(), &Object::new());
        Ok(())
    }


    pub fn create_dir_sync(&self, path: &str) -> std::io::Result<()> {
        let path = &self.path(path);
        if exists_sync(path)? {
            return Ok(());
        }
        match mkdir_sync(
            path,
            MkdirOptions {
                recursive: true,
            },
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.already_exists() {
                    Ok(())
                } else {
                    Err(std::io::Error::other(format!("failed to create dir {e:?}")))
                }
            }
        }
    }

    fn read_dir_sync(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        let Some(entries) = read_dir_sync(&self.path(path))? else {
            return Ok(None);
        };
        Ok(Some(entries
            .iter()
            .map(|name| {
                let uri = Path::new(path.trim_start_matches(&self.workspace_folder));
                uri.join(name).as_uri()
            })
            .collect()))
    }
}


#[wasm_bindgen]
#[derive(serde::Serialize, serde::Deserialize)]
struct MkdirOptions {
    pub recursive: bool,
}


#[async_trait(? Send)]
impl FileSystem for NodeFileSystem {
    async fn stat(&self, path: &str) -> std::io::Result<Option<Stat>> {
        let path = self.path(path);
        if exists_sync(&path)? {
            let stats = lstat_sync(&path)?;

            Ok(Some(Stat {
                ty: if stats.is_file() {
                    StatType::File
                } else {
                    StatType::Dir
                },
                size: if stats.is_file() {
                    stats.size() as u64
                } else {
                    self.read_dir(&path).await?.unwrap_or_default().len() as u64
                },
                create_time: (stats.c_time_ms() / 1000) as u64,
                update_time: (stats.m_time_ms() / 1000) as u64,
            }))
        } else {
            Ok(None)
        }
    }

    #[inline(always)]
    async fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        self.write_sync(path, buf)
    }

    #[inline(always)]
    async fn create_dir(&self, path: &str) -> std::io::Result<()> {
        self.create_dir_sync(path)
    }

    async fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        let path = self.path(path);
        if exists_sync(&path)? {
            let buffer = read_file_sync(&path)
                .map_err(|e| std::io::Error::other(format!("failed read file: {e:?}")))?;

            if buffer.is_string() {
                Ok(Some(buffer.as_string().unwrap().into_bytes()))
            } else {
                let buffer: Uint8Array = buffer.unchecked_into();
                let buffer = buffer.to_vec();
                Ok(Some(buffer))
            }
        } else {
            Ok(None)
        }
    }

    #[inline(always)]
    async fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        self.read_dir_sync(path)
    }

    #[inline(always)]
    async fn delete(&self, path: &str) -> std::io::Result<()> {
        let entry_path = self.path(path);
        rm_recursive(entry_path)
    }
}

#[cfg(test)]
mod tests {
    use meltos_tvc::file_system::FileSystem;
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::sleep::sleep_ms;
    use crate::tests::node_fs;

    #[wasm_bindgen_test]
    async fn read_root_dir() {
        let fs = node_fs();
        fs.delete("dir1").await.unwrap();
        fs.create_dir_sync("dir1").unwrap();
        let dir = fs.read_dir("dir1").await.unwrap();
        assert_eq!(dir.unwrap().len(), 0);
    }

    #[wasm_bindgen_test]
    async fn read_root_dir_with_files() {
        let fs = node_fs();
        fs.delete("dir2").await.unwrap();
        fs.write_file("dir2/1.txt", b"1").await.unwrap();
        fs.write_file("dir2/2.txt", b"1").await.unwrap();
        fs.write_file("dir2/3.txt", b"1").await.unwrap();

        let dir = fs.read_dir("dir2").await.unwrap();
        assert_eq!(dir.unwrap().len(), 3);
    }

    #[wasm_bindgen_test]
    async fn create_src_dir() {
        let fs = node_fs();
        fs.delete("dir3").await.unwrap();
        fs.create_dir("dir3").await.unwrap();
        let dir = fs.try_read_dir("dir3").await.unwrap();
        assert_eq!(dir.len(), 0);

        fs.write_file("dir3/hello.txt", b"hello").await.unwrap();
        let src = fs.try_read_dir("dir3").await.unwrap();
        assert_eq!(src.len(), 1);
    }

    #[wasm_bindgen_test]
    async fn create_parent_dirs() {
        let fs = node_fs();
        fs.delete("dir4").await.unwrap();
        fs.write_sync("dir4/hello.txt", b"hello").unwrap();
        fs.write_sync("dir4/hello2.txt", b"hello").unwrap();
        fs.write_sync("dir4/hello3.txt", b"hello").unwrap();

        let dist = fs.try_read_dir("dir4").await.unwrap();
        assert_eq!(dist.len(), 3);
    }

    #[wasm_bindgen_test]
    async fn read_hello_world() -> std::io::Result<()> {
        let fs = node_fs();
        fs.delete("dir5").await.unwrap();
        fs.write_sync("dir5/hello.txt", b"hello world")?;
        fs.write_sync("dir5/dist/hello.txt", b"hello world")?;
        fs.write_sync("dir5/dist/sample/hello.txt", b"hello world")?;

        let buf = fs.read_file("dir5/hello.txt").await.unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
        let buf = fs.read_file("dir5//dist/hello.txt").await.unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
        let buf = fs.read_file("dir5/dist/sample/hello.txt").await.unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
        Ok(())
    }

    #[wasm_bindgen_test]
    async fn read_file_start_with_period() {
        let fs = node_fs();
        fs.delete("dir6").await.unwrap();
        fs.write_sync("dir6/hello.txt", b"hello world").unwrap();
        fs.write_sync("dir6/dist/hello.txt", b"hello world").unwrap();
        fs.write_sync("dir6/dist/sample/hello.txt", b"hello world").unwrap();

        let buf = fs.read_file("dir6/hello.txt").await.unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
        let buf = fs.read_file("dir6/dist/hello.txt").await.unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
        let buf = fs.read_file("dir6/dist/sample/hello.txt").await.unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
    }

    #[wasm_bindgen_test]
    async fn delete_file() {
        let fs = node_fs();
        fs.delete("dir7").await.unwrap();
        fs.write_sync("dir7/hello.txt", b"hello world").unwrap();
        fs.delete("dir7/hello.txt").await.unwrap();

        assert_eq!(fs.read_file("dir7/hello.txt").await.unwrap(), None);
    }

    #[wasm_bindgen_test]
    async fn delete_dir() {
        let fs = node_fs();
        fs.delete("dir8").await.unwrap();
        fs.create_dir("dir8/src").await.unwrap();
        fs.write_file("dir8/src/hello.txt", b"hello").await.unwrap();

        fs.write_sync("dir8/dist/hello.txt", b"hello").unwrap();
        fs.write_sync("dir8/dist/sample/sample.js", b"console.log(`sample`)").unwrap();

        assert_eq!(fs.read_dir("dir8/src").await.unwrap().unwrap().len(), 1);
        assert_eq!(fs.read_dir("dir8/dist/sample").await.unwrap().unwrap().len(), 1);

        fs.delete("dir8/src").await.unwrap();
        assert!(fs.read_dir("dir8/src").await.unwrap().is_none());
        assert_eq!(fs.read_dir("dir8/dist/sample").await.unwrap().unwrap().len(), 1);
        assert_eq!(fs.try_read_dir("dir8/dist").await.unwrap().len(), 2);

        fs.delete("dir8/dist/sample").await.unwrap();
        assert!(fs.read_dir("dir8/src").await.unwrap().is_none());
        assert!(fs.read_dir("dir8/dist/sample").await.unwrap().is_none());
        assert_eq!(fs.try_read_dir("dir8/dist").await.unwrap().len(), 1);
    }

    #[wasm_bindgen_test]
    async fn all_files_with_in_children() {
        let fs = node_fs();
        fs.delete("dir9").await.unwrap();
        fs.write_sync("dir9/hello1.txt", b"hello").unwrap();
        fs.write_sync("dir9/hello2.txt", b"hello").unwrap();
        fs.write_sync("dir9/hello3.txt", b"hello").unwrap();

        let mut files = fs.all_files_in("dir9").await.unwrap();
        files.sort();
        assert_eq!(
            files,
            vec![
                "dir9/hello1.txt".to_string(),
                "dir9/hello2.txt".to_string(),
                "dir9/hello3.txt".to_string(),
            ]
        );
    }

    #[wasm_bindgen_test]
    async fn all_files_recursive() {
        let fs = node_fs();
        fs.delete("dir10").await.unwrap();
        fs.write_sync("dir10/hello1.txt", b"hello").unwrap();
        fs.write_sync("dir10/src/hello2.txt", b"hello").unwrap();
        fs.write_sync("dir10/src/dist/hello3.txt", b"hello").unwrap();

        let mut files = fs.all_files_in("dir10").await.unwrap();
        files.sort();
        assert_eq!(
            files,
            vec![
                "dir10/hello1.txt".to_string(),
                "dir10/src/dist/hello3.txt".to_string(),
                "dir10/src/hello2.txt".to_string(),
            ]
        );
    }

    #[wasm_bindgen_test]
    async fn all_files_relative_to_src() {
        let fs = node_fs();
        fs.delete("dir11").await.unwrap();
        fs.write_sync("dir11/hello1.txt", b"hello").unwrap();
        fs.write_sync("dir11/src/hello2.txt", b"hello").unwrap();
        fs.write_sync("dir11/src/dist/hello3.txt", b"hello").unwrap();

        let mut files = fs.all_files_in("dir11/src").await.unwrap();
        files.sort();
        assert_eq!(
            files,
            vec![
                "dir11/src/dist/hello3.txt".to_string(),
                "dir11/src/hello2.txt".to_string(),
            ]
        );
    }


    #[wasm_bindgen_test]
    async fn all_files_specified_direct_file_uri() {
        let fs = node_fs();
        fs.delete("dir12").await.unwrap();
        fs.write_sync("dir12/hello1.txt", b"hello").unwrap();

        let files = fs.all_files_in("dir12/hello1.txt").await.unwrap();
        assert_eq!(files, vec!["dir12/hello1.txt".to_string()]);
    }

    #[wasm_bindgen_test]
    async fn return_none_if_not_exists_entry() {
        let fs = node_fs();
        fs.delete("dir13").await.unwrap();
        fs.create_dir("dir13/src").await.unwrap();
        let stat = fs.stat("dir13/hello.txt").await.unwrap();
        assert_eq!(stat, None);
        let stat = fs.stat("dir13/src/hello.txt").await.unwrap();
        assert_eq!(stat, None);
    }

    #[wasm_bindgen_test]
    async fn stat_file() {
        let fs = node_fs();
        fs.delete("dir14").await.unwrap();
        fs.write_file("dir14/src/hello.txt", b"hello").await.unwrap();
        let stat = fs.stat("dir14/src/hello.txt").await.unwrap().unwrap();
        assert!(stat.is_file());
        assert_eq!(stat.size, b"hello".len() as u64);
    }

    #[wasm_bindgen_test]
    async fn stat_dir() {
        let fs = node_fs();
        fs.delete("dir15").await.unwrap();
        fs.create_dir("dir15/src").await.unwrap();
        let stat = fs.stat("dir15/src").await.unwrap().unwrap();
        assert!(stat.is_dir());
        assert_eq!(stat.size, 0);
    }

    #[wasm_bindgen_test]
    async fn update_dir_stat() {
        let fs = node_fs();
        fs.delete("dir16").await.unwrap();
        fs.create_dir("dir16/src").await.unwrap();

        fs.create_dir("dir16/src/dist").await.unwrap();
        let stat = fs.stat("dir16/src").await.unwrap().unwrap();
        assert_eq!(stat.size, 1);

        fs.write_file("dir16/src/hello.txt", b"hello world").await.unwrap();
        let stat = fs.stat("dir16/src").await.unwrap().unwrap();
        assert_eq!(stat.size, 2);
    }

    #[wasm_bindgen_test]
    async fn update_file_stat() {
        let fs = node_fs();
        fs.delete("dir17").await.unwrap();
        fs.write_file("dir17/src/hello.txt", b"hello world").await.unwrap();
        let stat1 = fs.stat("dir17/src/hello.txt").await.unwrap().unwrap();
        sleep_ms(1003).await;

        fs.write_file("dir17/src/hello.txt", b"hello").await.unwrap();
        let stat2 = fs.stat("dir17/src/hello.txt").await.unwrap().unwrap();

        assert_eq!(stat2.size, b"hello".len() as u64);

        assert!(stat1.update_time < stat2.update_time);
    }

    #[wasm_bindgen_test]
    async fn read() {
        let buf1 = [0, 1, 2, 3];
        let buf2 = [5, 6, 7, 8];
        let fs = node_fs();
        fs.delete("dir18").await.unwrap();

        fs.write_file("dir18/buf1", &buf1).await.unwrap();
        fs.write_file("dir18/buf2", &buf2).await.unwrap();
        assert_eq!(fs.read_file("dir18/buf1").await.unwrap().unwrap(), buf1.to_vec());
        assert_eq!(fs.read_file("dir18/buf2").await.unwrap().unwrap(), buf2.to_vec());
    }
}
