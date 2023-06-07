use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct VirtualFolderReference {
    pub name: String,
    pub virtual_path: String,
    pub quota_size: i64,
    pub quota_files: i32,
}
