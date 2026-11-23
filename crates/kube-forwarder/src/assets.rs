//! Assets embedded into the binary.
//!
//! `icons/*.svg` are monochrome icons (rendered with the current text color). Lucide icons
//! are used by gpui-component widgets, `icons/kf-*.svg` come from the original application.

use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

macro_rules! embed {
    ($($path:literal),* $(,)?) => {
        &[$(($path, include_bytes!(concat!("../assets/", $path)) as &[u8])),*]
    };
}

const FILES: &[(&str, &[u8])] = embed![
    "icons/check.svg",
    "icons/chevron-down.svg",
    "icons/chevron-right.svg",
    "icons/chevron-up.svg",
    "icons/chevrons-up-down.svg",
    "icons/circle-check.svg",
    "icons/circle-x.svg",
    "icons/close.svg",
    "icons/external-link.svg",
    "icons/eye.svg",
    "icons/inbox.svg",
    "icons/info.svg",
    "icons/loader.svg",
    "icons/loader-circle.svg",
    "icons/minus.svg",
    "icons/plus.svg",
    "icons/search.svg",
    "icons/triangle-alert.svg",
    "icons/kf-arrow-back.svg",
    "icons/kf-arrow-dropdown.svg",
    "icons/kf-arrow-thin-right.svg",
    "icons/kf-cross.svg",
    "icons/kf-dots.svg",
    "icons/kf-magnifier.svg",
    "icons/kf-pause.svg",
    "icons/kf-play.svg",
    "icons/kf-warning.svg",
    "images/logo.svg",
    "images/pattern.png",
];

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(FILES
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, data)| Cow::Borrowed(*data)))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(FILES
            .iter()
            .filter(|(name, _)| name.starts_with(path))
            .map(|(name, _)| SharedString::from(*name))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_used_by_gpui_component_is_embedded() {
        use gpui_component::{IconName, IconNamed};
        for icon in [
            IconName::Check,
            IconName::ChevronDown,
            IconName::ChevronRight,
            IconName::CircleCheck,
            IconName::CircleX,
            IconName::Close,
            IconName::ExternalLink,
            IconName::Eye,
            IconName::Inbox,
            IconName::Info,
            IconName::Loader,
            IconName::Search,
            IconName::TriangleAlert,
        ] {
            let path = icon.path();
            assert!(Assets.load(&path).unwrap().is_some(), "{path} is missing");
        }
    }
}
