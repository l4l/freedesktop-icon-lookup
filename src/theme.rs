use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use tini::Ini;

use crate::{Directory, Error, Result};

#[cfg_attr(any(test, feature = "debug"), derive(Debug))]
pub(crate) struct Theme {
    path: PathBuf,
    icon_infos: HashMap<String, Vec<IconInfo>>,
    inherits: Vec<String>,
}

impl Theme {
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let ini = read_index(path)?;

        let inherits = ini
            .get::<String>("Icon Theme", "Inherits")
            .map(|x| x.split(',').map(|x| x.trim().to_string()).collect())
            .unwrap_or_default();
        let directory_names = ini
            .get_vec::<String>("Icon Theme", "Directories")
            .ok_or_else(|| Error::InvalidTheme {
                reason: Some(r#""[Icon Theme].Directories" has invalid value"#.to_string()),
            })?;
        let scaled_directory_names = ini.get_vec::<String>("Icon Theme", "ScaledDirectories");

        let mut icon_infos = HashMap::<_, Vec<_>>::new();
        for d in directory_names
            .into_iter()
            .chain(scaled_directory_names.into_iter().flatten())
        {
            let directory = if let Some(d) = Directory::read(&ini, &d) {
                d
            } else {
                continue;
            };

            crate::find_dir_icons(path.join(&d), |icon_name, path| {
                icon_infos
                    .entry(icon_name.into())
                    .or_default()
                    .push(IconInfo { path, directory });
            })?;
        }

        Ok(Self {
            path: path.into(),
            icon_infos,
            inherits,
        })
    }

    pub(crate) fn icon_search<'a>(
        &'a self,
        name: &'a str,
        searcher: impl FnOnce(&[IconInfo]) -> Option<usize>,
    ) -> Option<IconSearch<'a>> {
        let icons = self.icon_infos.get(name)?;
        searcher(icons).and_then(|idx| {
            Some(IconSearch {
                theme: self,
                info: icons.get(idx)?,
            })
        })
    }

    pub(crate) fn inherits(&self) -> &[String] {
        &self.inherits
    }
}

pub(crate) struct IconSearch<'a> {
    theme: &'a Theme,
    info: &'a IconInfo,
}

impl IconSearch<'_> {
    pub fn path(&self) -> PathBuf {
        self.theme.path.join(&self.info.path)
    }
}

/// Information about found icon
#[cfg_attr(any(test, feature = "debug"), derive(Debug))]
pub struct IconInfo {
    path: PathBuf,
    directory: Directory,
}

impl IconInfo {
    pub fn is_svg(&self) -> bool {
        self.path.extension() == Some(OsStr::new("svg"))
    }

    pub fn is_png(&self) -> bool {
        self.path.extension() == Some(OsStr::new("png"))
    }

    pub fn size(&self) -> u16 {
        self.directory.size()
    }

    pub fn scale(&self) -> u16 {
        self.directory.scale()
    }

    pub(crate) fn directory(&self) -> Directory {
        self.directory
    }
}

fn read_index<P: AsRef<Path>>(theme_path: P) -> Result<Ini> {
    let theme_path = theme_path.as_ref();
    let index_path = theme_path.join("index.theme");
    let mut f = std::fs::File::open(index_path).map_err(|source| Error::ThemeIndexMissing {
        path: theme_path.into(),
        source,
    })?;
    Ini::from_reader(&mut f).map_err(|source| Error::InvalidIndex {
        path: theme_path.into(),
        source,
    })
}
