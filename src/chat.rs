use valence::{client::misc::ChatMessage, prelude::*};

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_chat_message);
    }
}

fn handle_chat_message(
    mut clients: Query<&mut Client>,
    usernames: Query<&Username>,
    mut events: EventReader<ChatMessage>,
) {
    for event in events.iter() {
        let username = usernames
            .get(event.client)
            .map(|username| username.0.clone())
            .unwrap_or(String::from("?"));
        let msg = "<".into_text()
            + username.color(Color::GRAY)
            + "> "
            + event.message.as_ref().to_owned();
        for mut client in &mut clients {
            client.send_message(msg.clone());
        }
    }
}
