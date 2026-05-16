use std::sync::Mutex;

use rcon2mc::rcon_client::RconClient;

use poise::serenity_prelude as serenity;

// Most of this is taken from the Poise example code https://docs.rs/poise/latest/poise/

// User data, which is stored and accessible in all command invocations
struct Data {
    rcon_client: Mutex<RconClient>,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Whitelists your account to the Minecraft Sever
#[poise::command(slash_command)]
async fn minecraft(
    ctx: Context<'_>,
    #[description = "Your Minecraft username"]
    username: String,
) -> Result<(), Error> {
    // Minecraft usernames should only contain "a-z", "0-9", and "_" and are, at most, 16 characters
    // Check to make sure before sending the command through RCON to prevent against injection attacks
    if !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') || username.len() > 16 {
        ctx.say("Your username seems to contain invalid characters or is too long").await?;
        return Ok(());
    }

    let feedback =
        ctx.data().rcon_client
        .lock()
        .unwrap()
        .send(format!("whitelist add {}", username));
    match feedback {
        Ok(f) => {
            if f.contains("Added ") {
                let response = format!("\
Thank you for showing interest in our Minecraft server!
To Join, download [Prism Launcher](https://prismlauncher.org/) and our [modpack](https://docspace-9td180.onlyoffice.com/s/jCYdr8DxjpMXz2N).
The server address is `foss.snails.cfd`.
See a live, 3D map of the world at http://snails.cfd:8200/#world.
Your username `{}` should have been added to the whitelist. Please let us know if you have any issues.", username);
                println!("{f}");
                ctx.say(response).await?;
            }
            else {
                println!("{f}");
                ctx.say(f).await?;
            }
        }
        Err(_) => {
            ctx.say("There was a problem sending the `whitelist` command to the Minecraft server, the server might be offline.").await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = std::fs::read_to_string("DISCORD_TOKEN").expect("Please put your discord token in the DISCORD_TOKEN file in the root of the project");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![minecraft()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    // Connect to the Minecraft server as an RCON client
                    rcon_client: Mutex::new(
                        RconClient::builder()
                        .host("127.0.0.1".to_string())
                        .port(25575)
                        // This port is not forwarded, it should not be possible for outside entities to connect
                        .password("password".to_string())
                        .build().expect("failed to connect to server"))
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token.trim(), intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}