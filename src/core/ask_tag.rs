#[derive(Clone)]
#[allow(dead_code)]
pub struct AskOptionTag {
    tag: String,
    localized_prompt: String,
    default: Option<bool>,
}

impl AskOptionTag {
    pub fn new(tag: &str, localized_prompt: &str, default: Option<bool>) -> AskOptionTag {
        AskOptionTag {
            tag: tag.to_owned(),
            localized_prompt: localized_prompt.to_owned(),
            default,
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct AskCardTag {
    tag: String,
    localized_prompt: String,
}

impl AskCardTag {
    pub fn new(tag: &str, localized_prompt: &str) -> AskCardTag {
        AskCardTag {
            tag: tag.to_owned(),
            localized_prompt: localized_prompt.to_owned(),
        }
    }
}
