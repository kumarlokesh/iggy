use crate::binary::sender::Sender;
use crate::streaming::session::Session;
use crate::streaming::systems::system::SharedSystem;
use anyhow::Result;
use iggy::error::IggyError;
use iggy::users::create_user::CreateUser;
use tracing::debug;

pub async fn handle(
    command: &CreateUser,
    sender: &mut dyn Sender,
    session: &Session,
    system: &SharedSystem,
) -> Result<(), IggyError> {
    debug!("session: {session}, command: {command}");
    let mut system = system.write().await;
    system
        .create_user(
            session,
            &command.username,
            &command.password,
            command.status,
            command.permissions.clone(),
        )
        .await?;
    sender.send_empty_ok_response().await?;
    Ok(())
}
