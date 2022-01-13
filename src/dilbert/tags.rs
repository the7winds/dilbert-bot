use anyhow::Result;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Tag(String);

pub trait ParseTags {
    fn parse_tags(&self) -> Result<Vec<Tag>>;
}

impl ParseTags for &str {
    fn parse_tags(&self) -> Result<Vec<Tag>> {
        let tags = self
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| !w.is_empty())
            .map(|w| Tag(w.to_lowercase()))
            .collect::<Vec<Tag>>();

        if tags.is_empty() {
            Err(anyhow::anyhow!("No tags was parsed."))
        } else {
            Ok(tags)
        }
    }
}

impl ToString for Tag {
    fn to_string(&self) -> String {
        let Tag(tag) = self;
        tag.to_string()
    }
}
