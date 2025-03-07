use crate::Property;
use aws_sdk_greengrassv2::types::Deployment;

/// Greengrass Deployments.
#[derive(Clone, Debug)]
pub struct Deployments {
    /// Inner type.
    inner: Vec<Deployment>,
}

impl From<Vec<Deployment>> for Deployments {
    fn from(inner: Vec<Deployment>) -> Self {
        Self { inner }
    }
}

impl<'a> Property<'a> for Deployments {
    fn items(&self) -> Vec<Vec<String>> {
        self.inner
            .iter()
            .map(|item| {
                vec![
                    item.deployment_name.as_ref().unwrap().to_string(),
                    item.deployment_status.as_ref().unwrap().to_string(),
                    item.creation_timestamp.as_ref().unwrap().to_string(),
                ]
            })
            .collect()
    }
}
