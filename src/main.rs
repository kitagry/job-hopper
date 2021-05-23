use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::api::batch::v1beta1::{CronJob, JobTemplateSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{
    api::{Api, ListParams, PostParams},
    Client,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::json;
use std::convert::Infallible;
use warp::{Filter, Reply};

async fn list_cronjobs(k8s_client: Client) -> Result<impl Reply, Infallible> {
    let cronjobs: Api<CronJob> = Api::namespaced(k8s_client, "");
    let lp = ListParams::default().timeout(20);

    match cronjobs.list(&lp).await {
        Ok(cronjobs) => {
            let cronjobs: Vec<JobTemplateSpec> = cronjobs
                .iter()
                .map(|c| {
                    let mut t = c.spec.clone().unwrap().job_template;
                    let mut metadata = ObjectMeta::default();
                    metadata.name = Some(create_tmp_job_name(c.metadata.name.clone()));
                    metadata.namespace = c.metadata.namespace.clone();
                    t.metadata = Some(metadata);
                    t
                })
                .collect();
            Ok(warp::reply::with_status(
                warp::reply::json(&cronjobs),
                http::StatusCode::OK,
            ))
        }
        Err(_) => Ok(warp::reply::with_status(
            warp::reply::json(&json!({"message": "failed"})),
            http::StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

fn create_tmp_job_name(s: Option<String>) -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
    let rand_string = rand_string.to_lowercase();
    match s {
        Some(s) => format!("{}-{}", s, rand_string),
        None => format!("job-hopper-{}", rand_string),
    }
}

async fn create_job(k8s_client: Client, job: Job) -> Result<impl Reply, Infallible> {
    let jobs: Api<Job> = Api::namespaced(k8s_client, &job.metadata.namespace.clone().unwrap());
    let pp = PostParams::default();
    match jobs.create(&pp, &job).await {
        Ok(_) => Ok(warp::reply::with_status(
            warp::reply::json(&json!({"message": "ok"})),
            http::StatusCode::CREATED,
        )),
        Err(e) => {
            Ok(warp::reply::with_status(
            warp::reply::json(&json!({"message": e.to_string()})),
            http::StatusCode::BAD_REQUEST,
        ))},
    }
}

fn with_k8s(k8s_client: Client) -> impl Filter<Extract = (Client,), Error = Infallible> + Clone {
    warp::any().map(move || k8s_client.clone())
}

fn json_body() -> impl Filter<Extract = (Job,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    let list_cronjobs = warp::path!("cronjob")
        .and(with_k8s(client.clone()))
        .and_then(list_cronjobs);

    let create_job = warp::post()
        .and(warp::path("job"))
        .and(with_k8s(client.clone()))
        .and(json_body())
        .and_then(create_job);

    let routes = list_cronjobs.or(create_job);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}
