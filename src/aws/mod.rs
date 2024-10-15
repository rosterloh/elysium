/// Greengrass Core Devices.
pub mod devices;
// pub mod command;

use devices::Device;

use std::time::Duration;

use crate::tui::AppResult;
use aws_config::{BehaviorVersion, meta::region::RegionProviderChain, stalled_stream_protection::StalledStreamProtectionConfig};
use aws_types::{
    region::Region,
    sdk_config::SdkConfig
};
use aws_sdk_greengrassv2::Client; // meta::PKG_VERSION, Error

/// AWS information.
#[derive(Debug)]
pub struct AwsCloud {
    /// Local AWS config.
    shared_config: SdkConfig,
    /// Greengrass Core Devices.
    pub devices: Vec<Device>,
}

impl AwsCloud {
    /// Constructs a new instance.
    pub async fn new(
        profile: &str,
        region: &str,
    ) -> AppResult<Self> {
        let region_provider = RegionProviderChain::first_try(Region::new(region.to_owned()))
            .or_default_provider()
            .or_else(Region::new("eu-west-1"));

        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .stalled_stream_protection(
            StalledStreamProtectionConfig::enabled()
                .upload_enabled(false)
                .grace_period(Duration::from_secs(10))
                .build()
            )
            .profile_name(profile)
            .region(region_provider)
            .load()
            .await;

        Ok(Self {
            shared_config: shared_config,
            devices: Vec::new(),
        })
    }

    pub async fn load(&mut self) -> AppResult<()> {
        self.devices = self.get_core_devices().await?;
        Ok(())
    }

    async fn get_core_devices(&self) -> AppResult<Vec<Device>> {
        let mut items: Vec<Device> = Vec::new();

        let client = Client::new(&self.shared_config);
    
        let resp = client.list_core_devices()
            .into_paginator()
            .send()
            .try_collect()
            .await;
        
        if resp.is_err() {
            // let sdk_error = &resp.as_ref().unwrap_err();
            // if let SdkError::DispatchFailure { err, .. } = sdk_error {
            // }
            return Err(resp.unwrap_err().to_string().into());
        }
    
        for device in resp?.into_iter().flat_map(|x| x.core_devices.unwrap_or_default()) {
            items.push(Device {
                name: device.core_device_thing_name().unwrap_or_default().to_string(),
                status: device.status().unwrap().to_string(),
                last_status_update_timestamp: device.last_status_update_timestamp().unwrap().to_string(),
            });
        }
    
        items.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(items)
    }
}