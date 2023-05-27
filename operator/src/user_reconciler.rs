use crate::reconciler::{ContextData, Error};
use crds::SftpgoUser;
use kube::runtime::controller::Action;
use std::sync::Arc;

pub async fn reconcile_user(
    resource: Arc<SftpgoUser>,
    context: Arc<ContextData>,
) -> Result<Action, Error> {
    info!("Running user conciliation");

    Ok(Action::requeue(std::time::Duration::from_secs(15)))
}
