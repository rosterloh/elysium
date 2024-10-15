/// Representation of a Device.
#[derive(Clone, Debug, Default)]
pub struct Device {
    /// Name of the core device.
    pub name: String,
    /// Device status HEALTHY or UNHEALTHY.
    pub status: String,
    /// The time at which the core device's status last updated, expressed in ISO 8601 format.
    pub last_status_update_timestamp: String,
}
