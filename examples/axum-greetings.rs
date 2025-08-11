use {
    async_stream::stream,
    axum::{
        Router,
        response::{Html, IntoResponse, Sse},
        routing::get,
    },
    core::{convert::Infallible, error::Error, time::Duration},
    datastar::{axum::ReadSignals, prelude::PatchSignals},
    serde::{Deserialize, Serialize},
    tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", get(index))
        .route("/greetings", get(hello_world));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(include_str!("greetings.html"))
}

#[derive(Deserialize)]
pub struct Signals {
    pub delay: u64,
    pub title: Option<Title>,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub suffix: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub enum Title {
    Mr,
    Mrs,
    Ms,
    Dr,
    Sir,
    Jedi,
}

impl std::fmt::Display for Title {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Title::Mr => write!(f, "Mr."),
            Title::Mrs => write!(f, "Mrs."),
            Title::Ms => write!(f, "Ms."),
            Title::Dr => write!(f, "Dr."),
            Title::Sir => write!(f, "Sir"),
            Title::Jedi => write!(f, "Jedi"),
        }
    }
}

async fn hello_world(ReadSignals(signals): ReadSignals<Signals>) -> impl IntoResponse {
    Sse::new(stream! {
        yield Ok::<_, Infallible>(PatchSignals::new(r#"{"message": ""}"#.to_string()).into());

        let mut message: Vec<String> = Vec::with_capacity(10);
        message.push("Greetings, ".to_string());
        if let Some(title) = signals.title {
            message.push(format!("{} ", title));
        }
        message.push(format!("{} ", signals.first_name));
        if let Some(middle_name) = signals.middle_name {
            message.push(format!("{} ", middle_name));
        }
        message.push(signals.last_name);
        match signals.suffix {
            Some(suffix) => {
                println!("Some(suffix): {}", suffix);
                message.push(format!(" {}!", suffix));
            }
            None => message.push("!".to_string()),
        }

        for i in 0..=message.len() {
            let message_signal = format!(r#"{{"message":"{}"}}"#, &message[0..i].join(""));
            let patch = PatchSignals::new(message_signal);

            yield Ok::<_, Infallible>(patch.into());

            tokio::time::sleep(Duration::from_millis(signals.delay)).await;
        }
    })
}
