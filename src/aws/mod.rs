use std::{error::Error, time::Duration};

use aws_config::{
    BehaviorVersion,
    meta::region::RegionProviderChain,
    stalled_stream_protection::StalledStreamProtectionConfig
};
use aws_sdk_greengrassv2::{self, error::SdkError, types::Deployment};
use aws_sdk_iot::{self, types::GroupNameAndArn};
use aws_types::{region::Region, sdk_config::SdkConfig};
use color_eyre::Result;

use crate::aws::{
    deployments::Deployments,
    devices::{Device, Devices},
    groups::ThingGroups,
};

pub mod devices;
pub mod deployments;
pub mod groups;

/// Property for receiving information.
pub trait Property<'a> {
    /// Returns the items.
    fn items(&self) -> Vec<Vec<String>>;
}

/// AWS information.
#[derive(Debug)]
pub struct AwsCloud {
    /// Local AWS config.
    #[allow(dead_code)]
    shared_config: SdkConfig,
    /// Greengrass connection client.
    gg_client: aws_sdk_greengrassv2::Client,
    /// IoT Core connection client.
    iot_client: aws_sdk_iot::Client,
    /// Greengrass Core Devices.
    pub devices: Devices,
    /// Thing Groups.
    pub groups: ThingGroups,
    /// Greengrass Deployments.
    pub deployments: Deployments,
}

impl AwsCloud {
    /// Constructs a new instance.
    pub async fn new(
        profile: &str,
        region: &str,
    ) -> Result<Self, String> {
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

        let client = aws_sdk_greengrassv2::Client::new(&shared_config);

        // Test to see if we need to authenicate
        let result = client.list_components().max_results(1).send().await;
        if result.is_err() {
            let sdk_error = &result.as_ref().unwrap_err();
            match sdk_error {
                SdkError::DispatchFailure(e) => {
                    return Err(format!("Please authenticate with aws-cli: aws login. {:?}", e.as_connector_error()));
                }
                SdkError::ServiceError(e) => {
                    return Err(format!("Service Error: {:?}", e.err().source()));
                }
                _ => {
                    return Err(sdk_error.to_string());
                }
            }            
        }

        let iot_client = aws_sdk_iot::Client::new(&shared_config);

        Ok(Self {
            shared_config: shared_config,
            gg_client: client,
            iot_client: iot_client,
            devices: Devices::from(vec![]),
            groups: ThingGroups::from(vec![]),
            deployments: Deployments::from(vec![]),
        })
    }

    pub async fn load(&mut self) -> Result<()> {
        self.devices = self.get_core_devices().await?;
        self.groups = self.get_thing_groups().await?;
        self.deployments = self.get_deployments().await?;
        Ok(())
    }

    async fn get_core_devices(&self) -> Result<Devices> {
        let mut items: Vec<Device> = Vec::new();
    
        let resp = self.gg_client.list_core_devices()
            .into_paginator()
            .send()
            .try_collect()
            .await?;
        
        for device in resp.into_iter().flat_map(|x| x.core_devices.unwrap_or_default()) {
            let thing_name =  device.core_device_thing_name().unwrap_or_default().to_string();
            // let connectivity = client.get_connectivity_info().thing_name(&thing_name).send().await?;
            items.push(Device {
                name: thing_name,
                status: device.status().unwrap().to_string(),
                last_status_update_timestamp: device.last_status_update_timestamp().unwrap().to_string(),
            });
        }
    
        items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(Devices::from(items))
    }

    async fn get_thing_groups(&self) -> Result<ThingGroups> {
        let mut items: Vec<GroupNameAndArn> = Vec::new();

        let resp = self.iot_client.list_thing_groups()
            .into_paginator()
            .send()
            .try_collect()
            .await?;

        for group in resp.into_iter().flat_map(|x| x.thing_groups.unwrap_or_default()) {
            items.push(group);
        }

        items.sort_by(|a, b| a.group_name.as_ref().unwrap().to_lowercase().cmp(&b.group_name.as_ref().unwrap().to_lowercase()));

        Ok(ThingGroups::from(items))
    }

    async fn get_deployments(&self) -> Result<Deployments> {
        let mut items: Vec<Deployment> = Vec::new();

        let resp = self.gg_client.list_deployments()
            .history_filter(aws_sdk_greengrassv2::types::DeploymentHistoryFilter::LatestOnly)
            .send()
            .await?;

        for deployment in resp.deployments.unwrap() {
            if deployment.deployment_name.is_some() {
                items.push(deployment);
            }
        }

        items.sort_by(|a, b| a.deployment_name.as_ref().unwrap().to_lowercase().cmp(&b.deployment_name.as_ref().unwrap().to_lowercase()));

        Ok(Deployments::from(items))
    }
}