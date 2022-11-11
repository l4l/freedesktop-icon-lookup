use std::collections::HashMap;
use std::path::{Path, PathBuf};

use either::Either;

use crate::{Error, IconInfo, IconSearch, Result, Theme};

const DEFAULT_THEME: &str = "hicolor";

/// Icon cache, before lookups one may need to load required theme(s)
/// explicitly either with [Cache::load] or [Cache::load_default].
pub struct Cache {
    themes: HashMap<String, Vec<Theme>>,
    pixmaps: HashMap<String, PathBuf>,
}

/// Search parameters for [Cache::lookup_param] search.
pub struct LookupParam<'a> {
    name: &'a str,
    theme: Option<&'a str>,
    size: Option<u16>,
    scale: Option<u16>,
}

impl Cache {
    /// Creates new cache. Most of the lookups are to be failed at this point.
    /// Consider loading icons afterwards.
    pub fn new() -> Result<Self> {
        Ok(Self {
            themes: HashMap::new(),
            pixmaps: {
                let mut pixmaps = HashMap::new();
                crate::find_dir_icons("/usr/share/pixmaps", |icon_name, path| {
                    pixmaps.insert(icon_name.into(), path);
                })?;
                pixmaps
            },
        })
    }

    /// Returns iterator with a loaded themes.
    pub fn themes(&self) -> impl Iterator<Item = &str> + '_ {
        self.themes.keys().map(|s| s.as_str())
    }

    /// Load icons for default (HiColor) icon theme.
    pub fn load_default(&mut self) -> Result<()> {
        self.load(DEFAULT_THEME)
    }

    /// Load icons for specified icon theme.
    pub fn load(&mut self, theme: impl Into<String>) -> Result<()> {
        self.load_inner(theme, 0)
    }

    fn load_inner(&mut self, theme: impl Into<String>, depth: usize) -> Result<()> {
        let theme = theme.into();
        if self.themes.contains_key(&theme) {
            return Ok(());
        }

        // In case of cyclic inherits
        if depth > 10 {
            return Err(Error::CycleDetected);
        }

        for path in search_dirs() {
            let path = path.join(&theme);
            if path.exists() {
                let t = Theme::new(path)?;

                if let Some(inherits) = t.inherits() {
                    self.load_inner(inherits, depth + 1)?;
                }

                self.themes.entry(theme.clone()).or_default().push(t);
            }
        }

        Ok(())
    }

    /// Advanced icon lookup provides general solution over [Cache::lookup].
    /// Similarly it requires an icon `name` and optional `theme` to look for.
    /// Also a provided closure is called for a list of discovered [IconInfo]'s.
    /// Notice that icons list might be incomplete (e.g doesn't include inherited themes).
    pub fn lookup_advanced<'a, F>(
        &'a self,
        name: &str,
        theme: impl Into<Option<&'a str>>,
        f: F,
    ) -> Option<PathBuf>
    where
        F: FnMut(&[IconInfo]) -> Option<usize> + Copy,
    {
        self.lookup_themed(theme.into().unwrap_or(DEFAULT_THEME), name, f, 0)
            .map(|s| s.path())
            .or_else(|| self.pixmaps.get(name).cloned())
    }

    /// Icon lookup for a specified `name` and optional `theme` to look for.
    /// If theme is unspecified the default one is used.
    pub fn lookup<'a>(&'a self, name: &str, theme: impl Into<Option<&'a str>>) -> Option<PathBuf> {
        self.lookup_param(LookupParam::new(name).with_theme(theme.into()))
    }

    /// Icon lookup with a provided [LookupParam]. It works as described in
    /// [Freedesktop icon lookup spec](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html).
    pub fn lookup_param<'a>(&'a self, param: LookupParam<'a>) -> Option<PathBuf> {
        self.lookup_themed(
            param.theme.unwrap_or(DEFAULT_THEME),
            param.name,
            |infos| {
                let (icon_size, icon_scale) = (param.size(), param.scale());

                if let Some(idx) = infos
                    .iter()
                    .position(|i| i.directory().is_matches(icon_size, icon_scale))
                {
                    return Some(idx);
                }

                if let Some((idx, _)) = infos
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, i)| i.directory().size_distance(icon_size, icon_scale))
                {
                    return Some(idx);
                }

                None
            },
            0,
        )
        .map(|s| s.path())
        .or_else(|| self.pixmaps.get(param.name).cloned())
    }

    fn lookup_themed<'a, F>(
        &'a self,
        theme: &str,
        icon_name: &'a str,
        f: F,
        depth: usize,
    ) -> Option<IconSearch<'a>>
    where
        F: FnMut(&[IconInfo]) -> Option<usize> + Copy,
    {
        // In case of cyclic inherits
        if depth > 10 {
            return None;
        }

        let themes = self.themes.get(theme)?;
        for theme in themes {
            if let Some(search) = theme.icon_search(icon_name, f) {
                return Some(search);
            }
        }

        for theme in themes.iter().filter_map(|t| t.inherits()) {
            if let Some(search) = self.lookup_themed(theme, icon_name, f, depth + 1) {
                return Some(search);
            }
        }

        None
    }
}

impl<'a> LookupParam<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            theme: None,
            size: None,
            scale: None,
        }
    }

    pub fn with_theme(mut self, theme: Option<&'a str>) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_scale(mut self, scale: u16) -> Self {
        self.scale = Some(scale);
        self
    }

    fn size(&self) -> u16 {
        self.size.unwrap_or(48)
    }

    fn scale(&self) -> u16 {
        self.size.unwrap_or(1)
    }
}

fn search_dirs() -> impl Iterator<Item = PathBuf> {
    use std::iter::once;

    std::env::var("HOME")
        .map(|var| PathBuf::from(var).join(".icons"))
        .into_iter()
        .chain(if let Ok(dirs) = std::env::var("XDG_DATA_DIRS") {
            Either::Left(
                dirs.split(':')
                    .map(|s| Path::new(s).join("icons"))
                    .collect::<Vec<_>>()
                    .into_iter(),
            )
        } else {
            Either::Right(
                once(PathBuf::from("/usr/share/local/icons"))
                    .into_iter()
                    .chain(once(PathBuf::from("/usr/share/icons"))),
            )
        })
}

pub(crate) fn find_dir_icons<P, F>(path: P, mut f: F) -> Result<()>
where
    P: AsRef<Path>,
    F: FnMut(&str, PathBuf),
{
    let path = path.as_ref();

    fn filter_io_errors<T>(r: std::io::Result<T>) -> Result<Option<T>> {
        use std::io::ErrorKind;
        match r {
            Ok(v) => Ok(Some(v)),
            Err(e) if matches!(e.kind(), ErrorKind::PermissionDenied | ErrorKind::NotFound) => {
                Ok(None)
            }
            Err(source) => Err(Error::TraverseDir { source }),
        }
    }

    for e in filter_io_errors(std::fs::read_dir(path))?
        .into_iter()
        .flatten()
    {
        if let Some(entry) = filter_io_errors(e)? {
            if filter_io_errors(entry.file_type())?.map_or(true, |f| f.is_dir()) {
                continue;
            }

            let path = entry.path();
            if let Some(icon_name) = path.file_name().and_then(|s| s.to_str()) {
                let icon_name = &icon_name[0..icon_name.rfind('.').unwrap_or(0)];
                f(icon_name, entry.path());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_search_dirs() {
        std::env::remove_var("XDG_DATA_DIRS");
        std::env::set_var("HOME", "/tmp");
        // `/usr/share/pixmaps` handled separately as it doesn't have themes.
        assert_eq!(
            vec!["/tmp/.icons", "/usr/share/local/icons", "/usr/share/icons"],
            search_dirs()
                .map(|p| p.to_str().unwrap().to_string())
                .collect::<Vec<_>>()
        );
    }
}
