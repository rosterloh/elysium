use crate::Property;

/// Representation of a Device.
#[derive(Clone, Debug, Default)]
pub struct Device {
    /// Name of the core device.
    pub name: String,
    /// Whether the IoT Core things is currently connected.
    pub is_connected: bool,
    /// Device status HEALTHY or UNHEALTHY.
    pub status: String,
    /// The time at which the core device's status last updated, expressed in ISO 8601 format.
    pub last_status_update_timestamp: String,
}

/// Greengrass Core devices wrapper.
#[derive(Clone, Debug)]
pub struct Devices {
    /// Inner type.
    inner: Vec<Device>,
}

impl From<Vec<Device>> for Devices {
    fn from(inner: Vec<Device>) -> Self {
        Self { inner }
    }
}

impl<'a> Property<'a> for Devices {
    fn items(&self) -> Vec<Vec<String>> {
        self.inner
            .iter()
            .map(|item| {
                vec![
                    item.name.to_string(),
                    item.status.to_string(),
                    item.last_status_update_timestamp.to_string(),
                ]
            })
            .collect()
    }
}
