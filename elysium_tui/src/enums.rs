use strum::{Display, EnumCount, EnumIter, FromRepr};

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter, EnumCount, PartialEq, Debug)]
pub enum TabsEnum {
    #[default]
    #[strum(to_string = "Core Devices")]
    Devices,
    #[strum(to_string = "Deployments")]
    Deployments,
}

impl TabsEnum {
    #[allow(unused)]
    pub fn headers(&self) -> &[&str] {
        match self {
            // TabsEnum::Sdk => todo!(),
            TabsEnum::Devices => &[
                "Name", "Status", "Last Status Update",
            ],
            // TabsEnum::ThingGroups => &[
            //     "Name", "ARN"
            // ],
            TabsEnum::Deployments => &[
                "Name", "Status", "Created",
            ]
        }
    }
}