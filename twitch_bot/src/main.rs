use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};
use twitch_irc::message::ServerMessage;
use futures_util::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::main]
pub async fn main() {
    // Load configuration from environment variables
    let bot_nickname = "xxskoodaxx_";
    let oauth_token = "0f4h3pvx4ftu46z82u1g92papwqt8s";
    let channel_to_join = "tempest0r";
    let user_to_watch = "xxskoodyxx_";
    let message_to_send = "Skoody Bot Steht zu deinen Diensten!";

    // Create the configuration for the IRC client
    let config = ClientConfig::new_simple(StaticLoginCredentials::new(
        bot_nickname.to_string(),
        Some(oauth_token.to_string()),
    ));

    // Create the IRC client
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    // Clone the client for use in the message handling task
    let client_clone = client.clone();

    // Wrap the receiver in a stream
    let mut stream = UnboundedReceiverStream::new(incoming_messages);

    // Start a task to handle incoming messages
    let join_handle = tokio::spawn(async move {
        while let Some(message) = stream.next().await {
            println!("Received message: {:?}", message); // Log all messages for debugging
            if let ServerMessage::Privmsg(privmsg) = message {
                // Check if the message is from the user we are watching
                if privmsg.sender.login.as_str() == user_to_watch {
                    println!("User {} said: {}", privmsg.sender.login, privmsg.message_text);
                    // Send the response message
                    if let Err(e) = client_clone.say(channel_to_join.to_string(), message_to_send.to_string()).await {
                        eprintln!("Failed to send message: {}", e);
                    }
                }
            }
        }
    });

    // Join the specified channel
    if let Err(e) = client.join(channel_to_join.to_string()) {
        eprintln!("Failed to join channel: {}", e);
        // If we can't join, there's no point in continuing
        return;
    }

    println!("Bot is running and connected to channel #{}", channel_to_join);

    // Keep the bot running
    if let Err(e) = join_handle.await {
        eprintln!("Message handling task failed: {:?}", e);
    }
}
