use crate::users::{add_user, get_user, remove_user, UserStore, UsernameStore};
use chat_types::{ClientMessage, ServerMessage};
use tokio::sync::broadcast;
use uuid::Uuid;

pub async fn handle_message(
    users: &UserStore,
    usernames: &UsernameStore,
    broadcast_tx: &broadcast::Sender<ServerMessage>,
    user_id: Option<Uuid>,
    message: ClientMessage,
) -> Option<ServerMessage> {
    match message {
        ClientMessage::Join { username } => {
            handle_join(users, usernames, broadcast_tx, username).await
        }
        ClientMessage::Leave => {
            if let Some(user_id) = user_id {
                handle_leave(users, usernames, broadcast_tx, user_id).await
            } else {
                Some(ServerMessage::Error {
                    error: "Not joined to a chat room".to_string(),
                })
            }
        }
        ClientMessage::SendMessage { content } => {
            if let Some(user_id) = user_id {
                handle_send_message(users, broadcast_tx, user_id, content).await
            } else {
                Some(ServerMessage::Error {
                    error: "Must join a chat room first".to_string(),
                })
            }
        }
    }
}

async fn handle_join(
    users: &UserStore,
    usernames: &UsernameStore,
    broadcast_tx: &broadcast::Sender<ServerMessage>,
    username: String,
) -> Option<ServerMessage> {
    match add_user(users, usernames, username.clone()) {
        Ok(user_id) => {
            // Broadcast that user joined
            let join_notification = ServerMessage::UserJoined {
                username: username.clone(),
            };
            let _ = broadcast_tx.send(join_notification);

            Some(ServerMessage::JoinSuccess { user_id })
        }
        Err(error) => Some(ServerMessage::JoinError { error }),
    }
}

async fn handle_leave(
    users: &UserStore,
    usernames: &UsernameStore,
    broadcast_tx: &broadcast::Sender<ServerMessage>,
    user_id: Uuid,
) -> Option<ServerMessage> {
    if let Some(username) = remove_user(users, usernames, user_id) {
        // Broadcast that user left
        let leave_notification = ServerMessage::UserLeft { username };
        let _ = broadcast_tx.send(leave_notification);
    }
    None // No direct response for leave
}

async fn handle_send_message(
    users: &UserStore,
    broadcast_tx: &broadcast::Sender<ServerMessage>,
    user_id: Uuid,
    content: String,
) -> Option<ServerMessage> {
    if let Some(user) = get_user(users, user_id) {
        let message = ServerMessage::Message {
            username: user.username,
            content,
        };
        let _ = broadcast_tx.send(message);
        None // No direct response for successful message send
    } else {
        Some(ServerMessage::Error {
            error: "User not found".to_string(),
        })
    }
}
