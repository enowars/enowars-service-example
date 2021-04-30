use enochecker::async_trait;
use enochecker::{run_checker, Checker, CheckerError, CheckerRequest, CheckerResult};
use serde::{Deserialize, Serialize};

use mongodb::{
    bson::doc,
    options::{ClientOptions, StreamAddress},
    Client,
};

use tracing::{debug, info, trace_span, warn, Instrument};

mod notebook_client;
use notebook_client::NotebookClient;

struct NotebookChecker {
    db: Client,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NotebookUser {
    username: String,
    password: String,
    note_id: Option<String>,
    note: Option<String>,
    task_chain_id: String,
}

impl NotebookChecker {
    async fn new() -> Self {
        let client = Client::with_options(
            ClientOptions::builder()
                .hosts(vec![StreamAddress {
                    hostname: "localhost".into(),
                    port: Some(27017),
                }])
                .build(),
        )
        .expect("Failed to establish mongo-client");

        for db_name in client
            .list_database_names(None, None)
            .await
            .expect("Mongo conn failed")
        {
            println!("{}", db_name);
        }

        NotebookChecker { db: client }
    }

    async fn deploy_flag_to_note(&self, checker_request: &CheckerRequest) -> CheckerResult<()> {
        let mut client = NotebookClient::connect(checker_request).await?;
        let mut user = NotebookUser {
            username: "Kevin".to_string(),
            password: "Kevinspassword".to_string(),
            note_id: None,
            note: None,
            task_chain_id: checker_request.task_chain_id.clone(),
        };

        client.register(&user).await?;

        Ok(())
    }
}

#[async_trait]
impl Checker for NotebookChecker {
    const SERVICE_NAME: &'static str = "n0t3b00k";
    const FLAG_VARIANTS: u64 = 1;
    const NOISE_VARIANTS: u64 = 1;
    const HAVOC_VARIANTS: u64 = 1;

    async fn putflag(&self, checker_request: &CheckerRequest) -> CheckerResult<()> {
        match checker_request.variant_id {
            0 => self.deploy_flag_to_note(checker_request).await,
            _ => Err(CheckerError::InternalError("Invalid variantId")),
        }
        // self.db
        //     .database("daw")
        //     .collection("dawdw")
        //     .insert_one(
        //         NotebookChecker {
        //             username: "penis".to_owned(),
        //             password: "1234".to_owned(),
        //             unique_id: checker_request.task_chain_id.clone(),
        //         },
        //         None,
        //     )
        //     .await
        //     .expect("Database insert failed");
    }

    async fn getflag(&self, checker_request: &CheckerRequest) -> CheckerResult<()> {
        let _foo: NotebookUser = self
            .db
            .database("daw")
            .collection("dawdw")
            .find_one(doc! { "unique_id": &checker_request.task_chain_id }, None)
            .await
            .unwrap()
            .unwrap();
        Ok(())
    }

    async fn putnoise(&self, _checker_request: &CheckerRequest) -> CheckerResult<()> {
        // Tracing information https://docs.rs/tracing/
        async {
            debug!("Registration successful");
        }
        .instrument(trace_span!("REGISTER"))
        .await;
        // instrument async code

        trace_span!("LOGIN").in_scope(|| info!("LOGIN DEBUG-PRINT")); // use in_scope only for syncronous subsections

        warn!("(WARN) PUTNOISE LOGGING");
        info!("(INFO) PUTNOISE LOGGING");
        debug!("(DBUG) PUTNOISE LOGGING");
        Ok(())
    }

    async fn getnoise(&self, _checker_request: &CheckerRequest) -> CheckerResult<()> {
        Err(CheckerError::Mumble("This is supposed to be a message that hopefully wraps when displayed on the checker-website. I hope <pre></pre> elements automagically add line breaks, since I don't know what I'll do if they don't D:."))
    }

    async fn havoc(&self, _checker_request: &CheckerRequest) -> CheckerResult<()> {
        Ok(())
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run_checker(NotebookChecker::new().await, 5499).await
}
