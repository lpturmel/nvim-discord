use crate::DISCORD_APP_ID;
pub use discord_sdk as ds;
pub use tokio;

pub struct Client {
    pub discord: ds::Discord,
    pub user: ds::user::User,
    pub wheel: ds::wheel::Wheel,
}

pub async fn make_client(subs: ds::Subscriptions) -> Client {
    let (wheel, handler) = ds::wheel::Wheel::new(Box::new(|err| {
        eprintln!("error: {}", err);
    }));

    let mut user = wheel.user();

    let discord = ds::Discord::new(
        ds::DiscordApp::PlainId(DISCORD_APP_ID),
        subs,
        Box::new(handler),
    )
    .expect("unable to create discord client");
    user.0.changed().await.unwrap();

    let user = match &*user.0.borrow() {
        ds::wheel::UserState::Connected(user) => user.clone(),
        ds::wheel::UserState::Disconnected(err) => panic!("failed to connect to Discord: {}", err),
    };

    Client {
        discord,
        user,
        wheel,
    }
}
