use tini::Ini;

#[derive(Clone, Copy)]
#[cfg_attr(any(test, feature = "debug"), derive(Debug))]
pub(crate) struct Directory {
    size: u16,
    scale: u16,
    kind: DirectoryKind,
}

#[derive(Clone, Copy)]
#[cfg_attr(any(test, feature = "debug"), derive(Debug))]
pub enum DirectoryKind {
    Fixed,
    Scalable { min_size: u16, max_size: u16 },
    Threshold { threshold: u16 },
}

impl Directory {
    pub(crate) fn read(ini: &Ini, folder: &str) -> Option<Self> {
        let size = ini.get(folder, "Size")?;
        let scale = ini.get(folder, "Scale").unwrap_or(1);
        let kind = ini.get::<String>(folder, "Type")?;
        Some(Directory {
            size,
            scale,
            kind: match kind.as_str() {
                "Fixed" => DirectoryKind::Fixed,
                "Scalable" => DirectoryKind::Scalable {
                    min_size: ini.get(folder, "MinSize")?,
                    max_size: ini.get(folder, "MaxSize")?,
                },
                "Threshold" => DirectoryKind::Threshold {
                    threshold: ini.get(folder, "Threshold").unwrap_or(2),
                },
                _ => return None,
            },
        })
    }

    pub(crate) fn is_matches(&self, icon_size: u16, icon_scale: u16) -> bool {
        if self.scale() != icon_scale {
            return false;
        }

        match self.kind() {
            DirectoryKind::Fixed => self.size() == icon_size,
            DirectoryKind::Scalable { min_size, max_size } => {
                (min_size..=max_size).contains(&icon_size)
            }
            DirectoryKind::Threshold { threshold } => (self.size().saturating_sub(threshold)
                ..=self.size().saturating_add(threshold))
                .contains(&icon_size),
        }
    }

    fn size_distance_range(&self) -> (u16, u16) {
        match self.kind() {
            DirectoryKind::Fixed => {
                let scaled_size = self.size().saturating_mul(self.scale());
                (scaled_size, scaled_size)
            }
            DirectoryKind::Scalable { min_size, max_size } => (
                min_size.saturating_mul(self.scale()),
                max_size.saturating_mul(self.scale()),
            ),
            DirectoryKind::Threshold { threshold } => (
                self.size().saturating_sub(threshold),
                self.size().saturating_add(threshold),
            ),
        }
    }

    pub(crate) fn size_distance(&self, icon_size: u16, icon_scale: u16) -> u16 {
        let icon_scaled_size = icon_size * icon_scale;
        let (left, right) = self.size_distance_range();

        if left > icon_scaled_size {
            left - icon_scaled_size
        } else if right < icon_scaled_size {
            icon_scaled_size - right
        } else {
            0
        }
    }

    pub(crate) fn scale(&self) -> u16 {
        self.scale
    }

    pub(crate) fn size(&self) -> u16 {
        self.size
    }

    pub(crate) fn kind(&self) -> DirectoryKind {
        self.kind
    }
}
