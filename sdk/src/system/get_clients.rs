use crate::command::{CommandExecution, CommandPayload};
use crate::error::IggyError;
use crate::validatable::Validatable;
use crate::{bytes_serializable::BytesSerializable, command::CommandExecutionOrigin};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// `GetClients` command is used to get the information about all connected clients.
/// It has no additional payload.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GetClients {}

impl CommandPayload for GetClients {}
impl CommandExecutionOrigin for GetClients {
    fn get_command_execution_origin(&self) -> CommandExecution {
        CommandExecution::Direct
    }
}

impl Validatable<IggyError> for GetClients {
    fn validate(&self) -> Result<(), IggyError> {
        Ok(())
    }
}

impl BytesSerializable for GetClients {
    fn as_bytes(&self) -> Bytes {
        Bytes::new()
    }

    fn from_bytes(bytes: Bytes) -> Result<GetClients, IggyError> {
        if !bytes.is_empty() {
            return Err(IggyError::InvalidCommand);
        }

        let command = GetClients {};
        command.validate()?;
        Ok(GetClients {})
    }
}

impl Display for GetClients {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_serialized_as_empty_bytes() {
        let command = GetClients {};
        let bytes = command.as_bytes();
        assert!(bytes.is_empty());
    }

    #[test]
    fn should_be_deserialized_from_empty_bytes() {
        let command = GetClients::from_bytes(Bytes::new());
        assert!(command.is_ok());
    }

    #[test]
    fn should_not_be_deserialized_from_empty_bytes() {
        let command = GetClients::from_bytes(Bytes::from_static(&[0]));
        assert!(command.is_err());
    }
}
