use serde_derive::{Deserialize, Serialize};
use validator::{Validate};
use uuid::Uuid;

#[derive(Debug, Deserialize, Validate, Serialize, Clone)]
pub struct ShareListBody {
    #[serde(rename = "targetUserId")]
    pub target_user_id: Uuid,
}