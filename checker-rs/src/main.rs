use enochecker::result::{CheckerError, CheckerResult};
use enochecker::{
    async_trait, run_checker,
    tokio::{spawn, try_join},
    Checker, CheckerRequest,
};

use std::env;

use fake::{
    faker::internet::en::{FreeEmail, Password, SafeEmail, Username},
    Fake,
};
use mongodb::{
    bson::doc,
    options::{ClientOptions, StreamAddress},
    Client,
};
use rand::random;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn, Instrument};

mod notebook_client;
use notebook_client::NotebookClient;

const DB_NAME: &str = "N0t3b00kCheckerDB";

const HELP_TEXT: &str = "
This is a notebook service. Commands:
reg USER PW - Register new account
log USER PW - Login to account
set TEXT..... - Set a note
user  - List all users
list - List all notes
exit - Exit!
dump - Dump the database
get ID";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotebookUser {
    username: String,
    password: String,
    note_id: Option<String>,
    note: Option<String>,
    task_chain_id: String,
}

impl NotebookUser {
    fn gen_random(id: String) -> Self {
        let username: String = if rand::random() {
            if rand::random() {
                FreeEmail().fake()
            } else {
                SafeEmail().fake()
            }
        } else {
            Username().fake()
        };

        Self {
            username,
            password: Password(12..24).fake::<String>(),
            note_id: None,
            note: None,
            task_chain_id: id,
        }
    }
}

struct NotebookChecker {
    db: Client,
}

impl NotebookChecker {
    async fn new() -> Self {
        let client = Client::with_options(
            ClientOptions::builder()
                .hosts(vec![StreamAddress {
                    hostname: env::var("MONGO_HOST").unwrap_or("mongo".to_string()),
                    port: Some(
                        env::var("MONGO_PORT")
                            .unwrap_or("27017".to_string())
                            .parse()
                            .expect("MONGO_PORT is not valid!"),
                    ),
                }])
                .build(),
        )
        .expect("Failed to establish mongo-client");

        //TODO: insert index
        let index_creation_result = client
            .database(DB_NAME)
            .run_command(
                doc! {
                    "createIndexes": "users",
                    "indexes": [{"key": {"task_chain_id": "hashed"}, "name": "ChainIndex" }]
                },
                None,
            )
            .await
            .expect("Failed to create MongoDB Index");

        // No Logger is created yet so we'll just use println!
        println!("Mongo index created: {}", index_creation_result);

        for db_name in client
            .list_database_names(None, None)
            .await
            .expect("Mongo conn failed")
        {
            println!("{}", db_name);
        }

        NotebookChecker { db: client }
    }

    async fn store_user(&self, user: NotebookUser) -> CheckerResult<()> {
        self.db
            .database(DB_NAME)
            .collection("users")
            .insert_one(user, None)
            .await
            .map_err(|e| {
                error!("DB-Insert failed {:?}", e);
                CheckerError::InternalError("Checker-DB insert failed")
            })?;

        Ok(())
    }

    async fn find_user(&self, id: &str) -> CheckerResult<Option<NotebookUser>> {
        self.db
            .database(DB_NAME)
            .collection("users")
            .find_one(
                doc! {
                    "task_chain_id": id,
                },
                None,
            )
            .await
            .map_err(|e| {
                error!("DB-Find failed {:?}", e);
                CheckerError::InternalError("Checker-DB find failed")
            })
    }

    async fn deploy_flag_to_note(&self, checker_request: &CheckerRequest) -> CheckerResult<()> {
        let mut client = NotebookClient::connect(checker_request).await?;
        let user = NotebookUser::gen_random(checker_request.task_chain_id.clone());

        client.register(&user).await?;
        client.login(user).await?;
        client
            .set_note(checker_request.flag.as_ref().unwrap())
            .await?;

        self.store_user(client.user.clone().unwrap()).await?;
        Ok(())
    }

    async fn check_flag_note(&self, checker_request: &CheckerRequest) -> CheckerResult<()> {
        let user = self
            .find_user(&checker_request.task_chain_id)
            .await?
            .ok_or(CheckerError::Mumble("Could not find old user"))?;
        let mut client = NotebookClient::connect(checker_request).await?;

        client.login(user.clone()).await?;
        let note_id = user.note_id.as_ref().unwrap();

        let notes = client.get_notes().await?;
        if !notes.contains(note_id) {
            warn!("Flag-Id '{}' was not in list", note_id);
            return Err(CheckerError::Mumble("Flag note is not in list"));
        }

        let flag = client.get_note().await?;
        if flag != checker_request.flag.as_ref().unwrap().as_str() {
            warn!(
                "Expected flag: '{}', got '{}'",
                checker_request.flag.as_ref().unwrap(),
                &flag
            );
            return Err(CheckerError::Mumble("Note did not contain correct flag"));
        }

        Ok(())
    }

    async fn deploy_noise_to_note(&self, checker_request: &CheckerRequest) -> CheckerResult<()> {
        let mut client = NotebookClient::connect(checker_request).await?;
        let user = NotebookUser::gen_random(checker_request.task_chain_id.clone());

        client.register(&user).await?;
        client.login(user).await?;

        client.set_note("sjgorisdr").await?;

        self.store_user(client.user.unwrap()).await
    }

    async fn check_noise_note(&self, checker_request: &CheckerRequest) -> CheckerResult<()> {
        let user = self
            .find_user(&checker_request.task_chain_id)
            .await?
            .ok_or(CheckerError::Mumble("Could not find old user"))?;
        let mut client = NotebookClient::connect(checker_request).await?;

        client.login(user).await?;
        let noise = client.get_note().await?;

        if &noise != client.user.as_ref().unwrap().note.as_ref().unwrap() {
            warn!(
                "Expected noise note with content {}, instead got {}",
                client.user.unwrap().note.unwrap(),
                noise
            );
            return Err(CheckerError::Mumble("Invalid note retrieved"));
        }
        Ok(())
    }

    async fn havoc_help(&self, checker_request: &CheckerRequest) -> CheckerResult<()> {
        // Well this will mean that not all service-details will get test-coverage,
        // however both lines are way too similar to warrant separete variants
        let help_text = if random() {
            info!("Getting help in an authenticated state");
            let user = NotebookUser::gen_random(checker_request.task_chain_id.clone());
            let mut client = NotebookClient::connect(checker_request).await?;
            client.register(&user).await?;
            client.login(user).await?;
            client.get_help().await?
        } else {
            info!("Getting help immediately");
            let mut client = NotebookClient::connect(checker_request).await?;
            client.get_help().await?
        };

        if help_text.as_str() != HELP_TEXT {
            return Err(CheckerError::Mumble("Invalid Help"));
        }

        Ok(())
    }

    async fn havoc_user(&self, checker_request: &CheckerRequest) -> CheckerResult<()> {
        info!("Getting userlist immediately");
        let user = NotebookUser::gen_random(checker_request.task_chain_id.clone());
        let mut client = NotebookClient::connect(checker_request).await?;
        client.register(&user).await?;

        let user1 = user.clone();
        let request2 = (*checker_request).clone();

        // Test the userlist as the user itself
        let future_auth = async move {
            info!("Trying to get user list");
            client.login(user1).await?;
            client.get_users().await
        };

        // And as any other connected user
        let future_unauth = async move {
            info!("Trying to get user list");
            let mut client = NotebookClient::connect(&request2).await?;
            client.get_users().await
        };

        // Launch both tasks in parralel
        // without spawn both would be still on the same thread and as such unable to run simultaneously
        // but still execute concurrently
        // Downside are some lifetime issues which need to be resolved
        // Usually this can be done by giving ownership of necessary structs into the closure (meaning clone (or leak :P))
        let user_lists: (CheckerResult<_>, CheckerResult<_>) = try_join!(
            spawn(future_auth.instrument(tracing::trace_span!("USERS-Authenticated"))),
            spawn(future_unauth.instrument(tracing::trace_span!("USERS-Immediately"))),
        )
        .map_err(|e| {
            error!("Failed to run tasks in parallel {:?}", e);
            CheckerError::InternalError("Join Failed")
        })?;

        if !user_lists.0?.contains(&user.username) | !user_lists.1?.contains(&user.username) {
            return Err(CheckerError::Mumble("User missing from list"));
        }
        Ok(())
    }
}

#[async_trait]
impl Checker for NotebookChecker {
    const SERVICE_NAME: &'static str = "n0t3b00k";
    const FLAG_VARIANTS: u64 = 1;
    const NOISE_VARIANTS: u64 = 1;
    const HAVOC_VARIANTS: u64 = 2;

    async fn putflag(&self, checker_request: &CheckerRequest) -> CheckerResult<()> {
        match checker_request.variant_id {
            0 => self.deploy_flag_to_note(checker_request).await,
            _ => Err(CheckerError::InternalError("Invalid variantId")),
        }
    }

    async fn getflag(&self, checker_request: &CheckerRequest) -> CheckerResult<()> {
        match checker_request.variant_id {
            0 => self.check_flag_note(checker_request).await,
            _ => Err(CheckerError::InternalError("Invalid variantId")),
        }
    }

    async fn putnoise(&self, checker_request: &CheckerRequest) -> CheckerResult<()> {
        // Tracing information https://docs.rs/tracing/
        match checker_request.variant_id {
            0 => self.deploy_noise_to_note(checker_request).await,
            _ => Err(CheckerError::InternalError("Invalid variantId")),
        }
    }

    async fn getnoise(&self, checker_request: &CheckerRequest) -> CheckerResult<()> {
        match checker_request.variant_id {
            0 => self.check_noise_note(checker_request).await,
            _ => Err(CheckerError::InternalError("Invalid variantId")),
        }
    }

    async fn havoc(&self, checker_request: &CheckerRequest) -> CheckerResult<()> {
        match checker_request.variant_id {
            0 => self.havoc_help(checker_request).await,
            1 => self.havoc_user(checker_request).await,
            _ => Err(CheckerError::InternalError("Invalid variantId")),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run_checker(NotebookChecker::new().await, 5499).await
}
