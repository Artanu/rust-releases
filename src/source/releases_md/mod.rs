use crate::source::releases_md::dl::fetch_releases_md;
use crate::source::Document;
use crate::source::{FetchResources, Source};
use crate::{Channel, Release, ReleaseIndex, TResult};

pub(in crate::source::releases_md) mod dl;

pub struct ReleasesMd {
    source: Document,
}

impl ReleasesMd {
    #[cfg(test)]
    pub(crate) fn from_document(source: Document) -> Self {
        Self { source }
    }
}

impl Source for ReleasesMd {
    fn build_index(&self) -> TResult<ReleaseIndex> {
        let contents = self.source.load()?;
        let content = String::from_utf8(contents).map_err(ReleasesMdError::UnrecognizedText)?;

        let releases = content
            .lines()
            .filter(|s| s.starts_with("Version"))
            .filter_map(|s| {
                s.split_ascii_whitespace()
                    .nth(1)
                    .and_then(|s| semver::Version::parse(s).map(Release::new).ok())
            });

        Ok(releases.collect())
    }
}

impl FetchResources for ReleasesMd {
    fn fetch_channel(channel: Channel) -> TResult<Self> {
        if let Channel::Stable = channel {
            let source = fetch_releases_md()?;
            Ok(Self { source })
        } else {
            Err(ReleasesMdError::ChannelNotAvailable(channel).into())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReleasesMdError {
    #[error("Channel {0} is not available for the releases-md source type")]
    ChannelNotAvailable(Channel),

    #[error("{0}")]
    UnrecognizedText(#[from] std::string::FromUtf8Error),
}

#[cfg(test)]
mod tests {
    use crate::dl_test;
    use crate::source::releases_md::ReleasesMd;
    use crate::source::Document;
    use crate::source::FetchResources;
    use crate::{Channel, ReleaseIndex};
    use yare::parameterized;

    #[test]
    fn strategy_dist_index() {
        let path = [
            env!("CARGO_MANIFEST_DIR"),
            "/resources/releases_md/RELEASES.md",
        ]
        .join("");
        let strategy = ReleasesMd::from_document(Document::LocalPath(path.into()));
        let index = ReleaseIndex::from_source(strategy).unwrap();

        assert!(index.releases().len() > 50);
    }

    #[parameterized(
        beta = { Channel::Beta },
        nightly = { Channel::Nightly },
    )]
    fn fetch_unsupported_channel(channel: Channel) {
        dl_test!({
            let file = ReleasesMd::fetch_channel(channel);
            assert!(file.is_err());
        })
    }

    #[test]
    fn fetch_supported_channel() {
        dl_test!({
            let file = ReleasesMd::fetch_channel(Channel::Stable);
            assert!(file.is_ok());
        })
    }
}