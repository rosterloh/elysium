use crate::aws::Property;
use aws_sdk_iot::types::GroupNameAndArn;

/// Greengrass Deployments.
#[derive(Clone, Debug)]
pub struct ThingGroups {
    /// Inner type.
    inner: Vec<GroupNameAndArn>,
}

impl From<Vec<GroupNameAndArn>> for ThingGroups {
    fn from(inner: Vec<GroupNameAndArn>) -> Self {
        Self { inner }
    }
}

impl<'a> Property<'a> for ThingGroups {
    fn items(&self) -> Vec<Vec<String>> {
        self.inner
            .iter()
            .map(|item| {
                vec![
                    item.group_name.as_ref().unwrap().to_string(),
                    item.group_arn.as_ref().unwrap().to_string(),
                ]
            })
            .collect()
    }
}
