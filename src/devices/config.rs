use crate::tui::AppResult;
use aws_config::{BehaviorVersion, meta::region::RegionProviderChain, stalled_stream_protection::StalledStreamProtectionConfig};
use aws_sdk_greengrassv2::{config::Region, meta::PKG_VERSION, Client, Error};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    pub body: String,
    pub internal: bool,
}

pub async fn load(profile: &str, region: &str) -> AppResult<Vec<Item>> {
    // trace_dbg!("Loading devices from {} using {}", region, PKG_VERSION);

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
    let client = Client::new(&shared_config);

    let items = get_core_devices(&client).await?;

    Ok(items)
}

async fn get_core_devices(client: &Client) -> AppResult<Vec<Item>> {
    let mut items: Vec<Item> = Vec::new();

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
        items.push(Item {
            name: device.core_device_thing_name().unwrap_or_default().to_string(),
            body: format!("Status: {:?}, Last Updated: {:?}", device.status().unwrap(), device.last_status_update_timestamp().unwrap()),
            internal: false,
        });
    }

    items.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(items)
}