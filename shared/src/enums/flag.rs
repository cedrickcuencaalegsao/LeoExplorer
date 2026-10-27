use serde::{Deserialize, Serialize};
use crate::enums::fs_attribute::FsAttribute;
use crate::enums::permissions::Permissions;
use crate::enums::explorer_state::ExplorerState;
use crate::enums::cloud_status::CloudStatus;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Flag {
    Attribute(FsAttribute),
    Permission(Permissions),
    Explorer(ExplorerState),
    Cloud(CloudStatus),
}