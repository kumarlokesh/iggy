use crate::binary::mapper;
use crate::streaming::polling_consumer::PollingConsumer;
use crate::streaming::systems::messages::PollingArgs;
use crate::tpc::shard::shard::IggyShard;
use crate::tpc::shard::shard_frame::ShardResponse;
use bytes::Bytes;
use iggy::command::Command;
use iggy::consumer_groups::create_consumer_group::CreateConsumerGroup;
use iggy::consumer_groups::delete_consumer_group::DeleteConsumerGroup;
use iggy::consumer_groups::get_consumer_group::GetConsumerGroup;
use iggy::consumer_groups::join_consumer_group::JoinConsumerGroup;
use iggy::consumer_groups::leave_consumer_group::LeaveConsumerGroup;
use iggy::consumer_offsets::get_consumer_offset::GetConsumerOffset;
use iggy::consumer_offsets::store_consumer_offset::StoreConsumerOffset;
use iggy::error::IggyError;
use iggy::messages::poll_messages::PollMessages;
use iggy::messages::send_messages::SendMessages;
use iggy::partitions::create_partitions::CreatePartitions;
use iggy::personal_access_tokens::create_personal_access_token::CreatePersonalAccessToken;
use iggy::personal_access_tokens::delete_personal_access_token::DeletePersonalAccessToken;
use iggy::personal_access_tokens::login_with_personal_access_token::LoginWithPersonalAccessToken;
use iggy::streams::create_stream::CreateStream;
use iggy::streams::delete_stream::DeleteStream;
use iggy::streams::get_stream::GetStream;
use iggy::streams::purge_stream::PurgeStream;
use iggy::system::get_client::GetClient;
use iggy::topics::create_topic::CreateTopic;
use iggy::topics::delete_topic::DeleteTopic;
use iggy::topics::get_topics::GetTopics;
use iggy::topics::purge_topic::PurgeTopic;
use iggy::topics::update_topic::UpdateTopic;
use iggy::users::change_password::ChangePassword;
use iggy::users::create_user::CreateUser;
use iggy::users::delete_user::DeleteUser;
use iggy::users::get_user::GetUser;
use iggy::users::login_user::LoginUser;
use iggy::users::update_permissions::UpdatePermissions;
use iggy::users::update_user::UpdateUser;

impl IggyShard {
    pub async fn handle_command(&self, command: Command) -> Result<ShardResponse, IggyError> {
        //debug!("Handling command '{command}', session: {session}...");
        match command {
            Command::Ping(_) => Ok(ShardResponse::BinaryResponse(Bytes::new())),
            Command::GetStats(_) => {
                let stats = self.get_stats(session).await?;
                let bytes = mapper::map_stats(&stats);
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::GetMe(_) => {
                let client = self.get_client(session, session.client_id).await?;
                let bytes = mapper::map_client(&client).await;
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::GetClient(command) => {
                let GetClient { client_id } = command;
                let client = self.get_client(session, client_id).await?;
                let bytes = mapper::map_client(&client).await;
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::GetClients(_) => {
                let clients = self.get_clients(session).await?;
                let bytes = mapper::map_clients(&clients).await;
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::GetUser(command) => {
                let GetUser { user_id } = command;
                let user = self.find_user(session, &user_id).await?;
                let bytes = mapper::map_user(&user);
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::GetUsers(_) => {
                let users = self.get_users(session).await?;
                let bytes = mapper::map_users(&users);
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::CreateUser(command) => {
                let CreateUser {
                    username,
                    password,
                    status,
                    permissions,
                } = command;
                self.create_user(session, username, password, status, permissions)
                    .await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::DeleteUser(command) => {
                let DeleteUser { user_id } = command;
                self.delete_user(session, &user_id).await?;
            }
            Command::UpdateUser(command) => {
                let UpdateUser {
                    user_id,
                    username,
                    status,
                } = command;
                self.update_user(session, &user_id, username, status)
                    .await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::UpdatePermissions(command) => {
                let UpdatePermissions {
                    user_id,
                    permissions,
                } = command;
                self.update_permissions(session, &user_id, permissions)
                    .await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::ChangePassword(command) => {
                let ChangePassword {
                    user_id,
                    current_password,
                    new_password,
                } = command;
                self.change_password(session, &user_id, current_password, new_password)
                    .await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::LoginUser(command) => {
                let LoginUser { username, password } = command;
                let user = self.login_user(&username, &password, Some(session)).await?;
                let bytes = mapper::map_identity_info(user.id);
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::LogoutUser(_) => {
                self.logout_user(session).await?;
                session.clear_user_id();
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::GetPersonalAccessTokens(_) => {
                let personal_access_tokens = self.get_personal_access_tokens(session).await?;
                let bytes = mapper::map_personal_access_tokens(&personal_access_tokens);
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::CreatePersonalAccessToken(command) => {
                let CreatePersonalAccessToken { name, expiry } = command;
                let token = self
                    .create_personal_access_token(session, name, expiry)
                    .await?;
                let bytes = mapper::map_raw_pat(&token);
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::DeletePersonalAccessToken(command) => {
                let DeletePersonalAccessToken { name } = command;
                self.delete_personal_access_token(session, name).await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::LoginWithPersonalAccessToken(command) => {
                let LoginWithPersonalAccessToken { token } = command;
                let user = self
                    .login_with_personal_access_token(&command.token, Some(session))
                    .await?;
                let bytes = mapper::map_identity_info(user.id);
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::SendMessages(command) => {
                let SendMessages {
                    stream_id,
                    topic_id,
                    partitioning,
                    messages,
                } = command;
                self.append_messages(session, stream_id, topic_id, partitioning, messages)
                    .await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::PollMessages(command) => {
                let PollMessages {
                    stream_id,
                    topic_id,
                    partition_id,
                    strategy,
                    consumer,
                    count,
                    auto_commit,
                } = command;
                let consumer =
                    PollingConsumer::from_consumer(consumer, session.client_id, partition_id);
                let messages = self
                    .poll_messages(
                        session,
                        consumer,
                        &stream_id,
                        &topic_id,
                        PollingArgs::new(strategy, count, auto_commit),
                    )
                    .await?;
                let messages = mapper::map_polled_messages(&messages);
                Ok(ShardResponse::BinaryResponse(messages))
            }
            Command::GetConsumerOffset(command) => {
                let GetConsumerOffset {
                    stream_id,
                    topic_id,
                    partition_id,
                    consumer,
                } = command;
                let consumer =
                    PollingConsumer::from_consumer(consumer, session.client_id, partition_id);
                let offset = self
                    .get_consumer_offset(session, consumer, &stream_id, &topic_id)
                    .await?;
                let bytes = mapper::map_consumer_offset(&offset);
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::StoreConsumerOffset(command) => {
                let StoreConsumerOffset {
                    stream_id,
                    topic_id,
                    partition_id,
                    consumer,
                    offset,
                } = command;
                let consumer =
                    PollingConsumer::from_consumer(consumer, session.client_id, partition_id);
                self.store_consumer_offset(
                    session,
                    consumer,
                    &stream_id,
                    &topic_id,
                    command.offset,
                )
                .await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::GetStream(command) => {
                let GetStream { stream_id } = command;
                let stream = self.find_stream(session, &command.stream_id)?;
                let bytes = mapper::map_stream(stream).await;
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::GetStreams(_) => {
                let streams = self.get_streams(session).await?;
                let bytes = mapper::map_streams(&streams).await;
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::CreateStream(command) => {
                let CreateStream { stream_id, name } = command;
                self.create_stream(session, stream_id, name).await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::DeleteStream(command) => {
                let DeleteStream { stream_id } = command;
                self.delete_stream(session, &stream_id).await?;
            }
            Command::UpdateStream(command) => {
                let UpdateStream { stream_id, name } = command;
                self.update_stream(session, &stream_id, name).await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::PurgeStream(command) => {
                let PurgeStream { stream_id } = command;
                self.purge_stream(session, &stream_id).await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::GetTopic(command) => {
                let GetTopic {
                    stream_id,
                    topic_id,
                } = command;
                let topic = self.find_topic(session, &stream_id, &topic_id)?;
                let bytes = mapper::map_topic(&topic);
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::GetTopics(command) => {
                let GetTopics { stream_id } = command;
                let topics = self.find_topics(session, &stream_id).await?;
                let bytes = mapper::map_topics(&topics);
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::CreateTopic(command) => {
                let CreateTopic {
                    stream_id,
                    topic_id,
                    name,
                    partitions_count,
                    message_expiry,
                    compression_algorithm,
                    max_topic_size,
                    replication_factor,
                } = command;
                self.create_topic(
                    session,
                    &stream_id,
                    &topic_id,
                    name,
                    partitions_count,
                    message_expiry,
                    compression_algorithm,
                    max_topic_size,
                    replication_factor,
                )
                .await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::DeleteTopic(command) => {
                let DeleteTopic {
                    stream_id,
                    topic_id,
                } = command;
                self.delete_topic(session, &stream_id, &topic_id).await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::UpdateTopic(command) => {
                let UpdateTopic {
                    stream_id,
                    topic_id,
                    name,
                    message_expiry,
                    compression_algorithm,
                    max_topic_size,
                    replication_factor,
                } = command;
                self.update_topic(
                    session,
                    &stream_id,
                    &topic_id,
                    name,
                    message_expiry,
                    compression_algorithm,
                    max_topic_size,
                    replication_factor,
                )
                .await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::PurgeTopic(command) => {
                let PurgeTopic {
                    stream_id,
                    topic_id,
                } = command;
                self.purge_topic(session, &stream_id, &topic_id).await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::CreatePartitions(command) => {
                let CreatePartitions {
                    stream_id,
                    topic_id,
                    partitions_count: count,
                } = command;
                self.create_partitions(session, &stream_id, &topic_id, count)
                    .await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::DeletePartitions(command) => {
                let DeletePartitions {
                    stream_id,
                    topic_id,
                    partitions_count: count,
                } = command;
                self.delete_partitions(session, &stream_id, &topic_id, partitions_count)
                    .await?;
            }
            Command::GetConsumerGroup(command) => {
                let GetConsumerGroup {
                    stream_id,
                    topic_id,
                    group_id,
                } = command;
                let consumer_group =
                    self.get_consumer_group(session, &stream_id, &topic_id, &group_id)?;
                let consumer_group = consumer_group.read().await;
                let bytes = mapper::map_consumer_group(&consumer_group).await;
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::GetConsumerGroups(command) => {
                let GetConsumerGroups {
                    stream_id,
                    topic_id,
                } = command;
                let consumer_groups = system.get_consumer_groups(session, &stream_id, &topic_id)?;
                let bytes = mapper::map_consumer_groups(&consumer_groups).await;
                Ok(ShardResponse::BinaryResponse(bytes))
            }
            Command::CreateConsumerGroup(command) => {
                let CreateConsumerGroup {
                    stream_id,
                    topic_id,
                    group_id,
                    name,
                } = command;
                self.create_consumer_group(session, &stream_id, &topic_id, group_id, &name)
                    .await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::DeleteConsumerGroup(command) => {
                let DeleteConsumerGroup {
                    stream_id,
                    topic_id,
                    group_id,
                } = command;
                self.delete_consumer_group(session, &stream_id, &topic_id, &group_id)
                    .await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::JoinConsumerGroup(command) => {
                let JoinConsumerGroup {
                    stream_id,
                    topic_id,
                    group_id,
                } = command;
                self.join_consumer_group(session, &stream_id, &topic_id, &group_id)
                    .await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
            Command::LeaveConsumerGroup(command) => {
                let LeaveConsumerGroup {
                    stream_id,
                    topic_id,
                    group_id,
                } = command;
                self.leave_consumer_group(session, &stream_id, &topic_id, &group_id)
                    .await?;
                Ok(ShardResponse::BinaryResponse(Bytes::new()))
            }
        }
    }
}
