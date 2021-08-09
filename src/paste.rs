use crate::config::Config;
use crate::header::ContentDisposition;
use std::convert::TryFrom;
use std::fs::{self, File};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult, Write};
use std::path::PathBuf;
use std::str;
use url::Url;

/// Type of the data to store.
#[derive(Clone, Copy, Debug)]
pub enum PasteType {
    /// Any type of file.
    File,
    /// A file that only contains an URL.
    Url,
}

impl<'a> TryFrom<&'a ContentDisposition> for PasteType {
    type Error = ();
    fn try_from(content_disposition: &'a ContentDisposition) -> Result<Self, Self::Error> {
        if content_disposition.has_form_field("file") {
            Ok(Self::File)
        } else if content_disposition.has_form_field("url") {
            Ok(Self::Url)
        } else {
            Err(())
        }
    }
}

/// Representation of a single paste.
#[derive(Debug)]
pub struct Paste {
    /// Data to store.
    pub data: Vec<u8>,
    /// Type of the data.
    pub type_: PasteType,
}

impl Paste {
    /// Writes the bytes to a file in upload directory.
    ///
    /// - If `file_name` does not have an extension, it is replaced with [`default_extension`].
    /// - If `file_name` is "-", it is replaced with "stdin".
    /// - If [`random_url.enabled`] is `true`, `file_name` is replaced with a pet name or random string.
    ///
    /// [`default_extension`]: crate::config::PasteConfig::default_extension
    /// [`random_url.enabled`]: crate::random::RandomURLConfig::enabled
    pub fn store_file(&self, file_name: &str, config: &Config) -> IoResult<String> {
        let file_name = match PathBuf::from(file_name)
            .file_name()
            .map(|v| v.to_str())
            .flatten()
        {
            Some("-") => String::from("stdin"),
            Some(v) => v.to_string(),
            None => String::from("file"),
        };
        let mut path = config.server.upload_path.join(file_name);
        match path.clone().extension() {
            Some(extension) => {
                if let Some(file_name) = config.paste.random_url.generate() {
                    path.set_file_name(file_name);
                    path.set_extension(extension);
                }
            }
            None => {
                if let Some(file_name) = config.paste.random_url.generate() {
                    path.set_file_name(file_name);
                }
                path.set_extension(
                    infer::get(&self.data)
                        .map(|t| t.extension())
                        .unwrap_or(&config.paste.default_extension),
                );
            }
        }
        let mut buffer = File::create(&path)?;
        buffer.write_all(&self.data)?;
        Ok(path
            .file_name()
            .map(|v| v.to_string_lossy())
            .unwrap_or_default()
            .to_string())
    }

    /// Writes an URL to a file in upload directory.
    ///
    /// - Checks if the data is a valid URL.
    /// - If [`random_url.enabled`] is `true`, file name is set to a pet name or random string.
    ///
    /// [`random_url.enabled`]: crate::random::RandomURLConfig::enabled
    pub fn store_url(&self, config: &Config) -> IoResult<String> {
        let data = str::from_utf8(&self.data)
            .map_err(|e| IoError::new(IoErrorKind::Other, e.to_string()))?;
        let url = Url::parse(data).map_err(|e| IoError::new(IoErrorKind::Other, e.to_string()))?;
        let file_name = config
            .paste
            .random_url
            .generate()
            .unwrap_or_else(|| String::from("url"));
        let path = config.server.upload_path.join("url").join(&file_name);
        fs::write(&path, url.to_string())?;
        Ok(file_name)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::random::{RandomURLConfig, RandomURLType};
    use std::env;

    #[test]
    fn test_paste_data() -> IoResult<()> {
        let mut config = Config::default();
        config.server.upload_path = env::current_dir()?;
        config.paste.random_url = RandomURLConfig {
            enabled: true,
            words: Some(3),
            separator: Some(String::from("_")),
            type_: RandomURLType::PetName,
            ..RandomURLConfig::default()
        };
        let paste = Paste {
            data: vec![65, 66, 67],
            type_: PasteType::File,
        };
        let file_name = paste.store_file("test.txt", &config)?;
        assert_eq!("ABC", fs::read_to_string(&file_name)?);
        assert_eq!(
            Some("txt"),
            PathBuf::from(&file_name)
                .extension()
                .map(|v| v.to_str())
                .flatten()
        );
        fs::remove_file(file_name)?;

        config.paste.default_extension = String::from("bin");
        config.paste.random_url.enabled = false;
        config.paste.random_url = RandomURLConfig {
            enabled: true,
            length: Some(10),
            type_: RandomURLType::Alphanumeric,
            ..RandomURLConfig::default()
        };
        let paste = Paste {
            data: vec![120, 121, 122],
            type_: PasteType::File,
        };
        let file_name = paste.store_file("random", &config)?;
        assert_eq!("xyz", fs::read_to_string(&file_name)?);
        assert_eq!(
            Some("bin"),
            PathBuf::from(&file_name)
                .extension()
                .map(|v| v.to_str())
                .flatten()
        );
        fs::remove_file(file_name)?;

        config.paste.random_url.enabled = false;
        let paste = Paste {
            data: vec![116, 101, 115, 116],
            type_: PasteType::File,
        };
        let file_name = paste.store_file("test.file", &config)?;
        assert_eq!("test.file", &file_name);
        assert_eq!("test", fs::read_to_string(&file_name)?);
        fs::remove_file(file_name)?;

        fs::create_dir_all(config.server.upload_path.join("url"))?;

        config.paste.random_url.enabled = true;
        let url = String::from("https://orhun.dev/");
        let paste = Paste {
            data: url.as_bytes().to_vec(),
            type_: PasteType::Url,
        };
        let file_name = paste.store_url(&config)?;
        let file_path = config.server.upload_path.join("url").join(&file_name);
        assert_eq!(url, fs::read_to_string(&file_path)?);
        fs::remove_file(file_path)?;

        let url = String::from("testurl.com");
        let paste = Paste {
            data: url.as_bytes().to_vec(),
            type_: PasteType::Url,
        };
        assert!(paste.store_url(&config).is_err());

        fs::remove_dir(config.server.upload_path.join("url"))?;

        Ok(())
    }
}