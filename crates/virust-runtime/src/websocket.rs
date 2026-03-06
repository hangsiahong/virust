use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};

pub async fn ws_upgrade(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(result) = socket.recv().await {
        match result {
            Ok(Message::Text(text)) => {
                // Echo back for now
                let _ = socket.send(Message::Text(text)).await;
            }
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }
}