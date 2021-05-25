use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::api::batch::v1beta1::{CronJob, JobTemplateSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{
    api::{Api, ListParams, PostParams},
    Client,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::convert::Infallible;
use warp::{Filter, Reply};

#[derive(Deserialize, Serialize)]
struct ListCronJobParam {
    namespace: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct JobTemplate {
    cronjob_data: CronJobData,
    spec: JobSpec,
}

#[derive(Deserialize, Serialize)]
struct CronJobData {
    name: String,
    namespace: String,
}

#[derive(Deserialize, Serialize)]
struct JobSpec {
    containers: Vec<Container>,
}

#[derive(Deserialize, Serialize)]
struct Container {
    name: String,
    image: String,
    command: Vec<String>,
    args: Vec<String>,
    env: Vec<Env>,
}

#[derive(Deserialize, Serialize)]
struct Env {
    name: String,
    value: String,
}

impl JobTemplate {
    fn new(cj: CronJob) -> Self {
        JobTemplate {
            cronjob_data: CronJobData {
                name: cj.metadata.name.unwrap_or("".to_string()),
                namespace: cj.metadata.namespace.unwrap_or("".to_string()),
            },
            spec: JobSpec {
                containers: cj
                    .spec
                    .unwrap()
                    .job_template
                    .spec
                    .unwrap()
                    .template
                    .spec
                    .unwrap()
                    .containers
                    .iter()
                    .map(|c| Container::new(c.clone()))
                    .collect(),
            },
        }
    }
}

impl Container {
    fn new(c: k8s_openapi::api::core::v1::Container) -> Self {
        Container {
            name: c.name,
            image: c.image.unwrap_or("".to_string()),
            command: c.command.unwrap_or(Vec::new()),
            args: c.args.unwrap_or(Vec::new()),
            env: c
                .env
                .unwrap_or(Vec::new())
                .iter()
                .map(|e| Env::new(e.clone()))
                .collect(),
        }
    }
}

impl Env {
    fn new(e: k8s_openapi::api::core::v1::EnvVar) -> Self {
        Env {
            name: e.name,
            value: e.value.unwrap_or("".to_string()),
        }
    }
}

async fn list_cronjobs(
    k8s_client: Client,
    params: ListCronJobParam,
) -> Result<impl Reply, Infallible> {
    let cronjobs: Api<CronJob> =
        Api::namespaced(k8s_client, &params.namespace.unwrap_or("".to_string()));
    let lp = ListParams::default().timeout(20);

    match cronjobs.list(&lp).await {
        Ok(cronjobs) => {
            let cronjobs: Vec<JobTemplate> = cronjobs
                .iter()
                .map(|c| JobTemplate::new(c.clone()))
                .collect();
            Ok(warp::reply::with_status(
                warp::reply::json(&cronjobs),
                http::StatusCode::OK,
            ))
        }
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&json!({"message": e.to_string()})),
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
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&json!({"message": e.to_string()})),
            http::StatusCode::BAD_REQUEST,
        )),
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
    let list_cronjobs = warp::path!("api" / "cronjob")
        .and(with_k8s(client.clone()))
        .and(warp::query::<ListCronJobParam>())
        .and_then(list_cronjobs);

    let create_job = warp::post()
        .and(warp::path("api"))
        .and(warp::path("job"))
        .and(with_k8s(client.clone()))
        .and(json_body())
        .and_then(create_job);

    let routes = list_cronjobs.or(create_job);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}
