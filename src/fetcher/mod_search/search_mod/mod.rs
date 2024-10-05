use serde::Deserialize;
use serde_repr::Deserialize_repr;
use serde_with::{serde_as, DisplayFromStr};
use url::Url;

use super::mod_links::ModLinks;

pub mod display;

#[derive(Debug, Clone, Deserialize)]
pub struct SearchedMod {
    id: usize,
    name: String,
    slug: String,
    summary: String,
    links: ModLinks,
    #[serde(rename = "thumbsUpCount")]
    thumbs_up_count: usize,
    #[serde(rename = "downloadCount")]
    download_count: usize,
}

impl SearchedMod {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn thumbs_up_count(&self) -> usize {
        self.thumbs_up_count
    }

    pub fn download_count(&self) -> usize {
        self.download_count
    }

    pub fn links(&self) -> &ModLinks {
        &self.links
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn display(&self) -> display::ModBuilder {
        display::ModBuilder::new(self)
    }

    pub fn display_with_options(&self, options: display::ModOptions) -> display::ModBuilder {
        display::ModBuilder::from_options(self, options)
    }
}

impl PartialEq for SearchedMod {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for SearchedMod {}

impl PartialOrd for SearchedMod {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchedMod {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.download_count
            .cmp(&other.download_count)
            .then_with(|| self.thumbs_up_count.cmp(&other.thumbs_up_count))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModRelation {
    #[serde(rename = "modId")]
    id: usize,
    #[serde(rename = "relationType")]
    relation: RelationType,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize_repr)]
#[repr(u8)]
pub enum RelationType {
    EmbeddedLibrary = 1,
    OptionalDependency = 2,
    RequiredDependency = 3,
    Tool = 4,
    Incompatible = 5,
    Include = 6,
}

impl RelationType {
    /// Returns `true` if the relation type is [`RequiredDependency`].
    ///
    /// [`RequiredDependency`]: RelationType::RequiredDependency
    #[must_use]
    pub fn is_required_dependency(&self) -> bool {
        matches!(self, Self::RequiredDependency)
    }

    /// Returns `true` if the relation type is [`Incompatible`].
    ///
    /// [`Incompatible`]: RelationType::Incompatible
    #[must_use]
    pub fn is_incompatible(&self) -> bool {
        matches!(self, Self::Incompatible)
    }

    /// Returns `true` if the relation type is [`EmbeddedLibrary`].
    ///
    /// [`EmbeddedLibrary`]: RelationType::EmbeddedLibrary
    #[must_use]
    pub fn is_embedded_library(&self) -> bool {
        matches!(self, Self::EmbeddedLibrary)
    }

    pub fn is_needed(&self) -> bool {
        self.is_required_dependency() || self.is_incompatible() || self.is_embedded_library()
    }
}

impl ModRelation {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn relation(&self) -> RelationType {
        self.relation
    }
}

#[allow(unused)]
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct ModFile {
    #[serde(rename = "fileName")]
    file_name: String,
    #[serde(rename = "downloadCount")]
    download_count: usize,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "downloadUrl")]
    url: Url,
    #[serde(rename = "gameVersions")]
    versions: Vec<String>,
    #[serde(rename = "dependencies")]
    pub relations: Vec<ModRelation>,
}

impl ModFile {
    pub fn versions(&self) -> &[String] {
        &self.versions
    }
}
