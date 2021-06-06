use async_trait::async_trait;
use job_hopper::model::{Container, JobTemplate};
use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::api::batch::v1beta1::CronJob;
use k8s_openapi::api::core::v1::{Container as K8sContainer, PodSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{
    api::{Api, ListParams, ObjectList, PostParams},
    Client,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::convert::Infallible;
use warp::{Filter, Reply};

#[async_trait]
trait K8sClient {
    async fn list_cronjobs(&self, namespace: &str) -> Result<ObjectList<CronJob>, kube::Error>;
    async fn get_cronjob(&self, namespace: &str, name: &str) -> Result<CronJob, kube::Error>;
    async fn create_job(&self, namespace: &str, job: &Job) -> Result<Job, kube::Error>;
}

#[derive(Clone)]
struct K8sClientImpl(Client);

#[async_trait]
impl K8sClient for K8sClientImpl {
    async fn list_cronjobs(&self, namespace: &str) -> Result<ObjectList<CronJob>, kube::Error> {
        let cronjobs: Api<CronJob> = Api::namespaced(self.0.clone(), namespace);
        let lp = ListParams::default().timeout(20);
        cronjobs.list(&lp).await
    }

    async fn get_cronjob(&self, namespace: &str, name: &str) -> Result<CronJob, kube::Error> {
        let cronjobs: Api<CronJob> = Api::namespaced(self.0.clone(), namespace);
        cronjobs.get(name).await
    }

    async fn create_job(&self, namespace: &str, job: &Job) -> Result<Job, kube::Error> {
        let jobs: Api<Job> = Api::namespaced(self.0.clone(), namespace);
        let pp = PostParams::default();
        jobs.create(&pp, job).await
    }
}

#[derive(Deserialize, Serialize)]
struct ListCronJobParam {
    namespace: Option<String>,
}

async fn list_cronjobs<T: K8sClient>(
    k8s_client: T,
    params: ListCronJobParam,
) -> Result<impl Reply, Infallible> {
    match k8s_client
        .list_cronjobs(&params.namespace.unwrap_or_else(|| "".to_string()))
        .await
    {
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

fn create_tmp_job_name(s: String) -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
    let rand_string = rand_string.to_lowercase();
    match s.len() {
        0 => format!("job-hopper-{}", rand_string),
        _ => format!("{}-{}", s, rand_string),
    }
}

async fn create_job<T: K8sClient>(
    k8s_client: T,
    job: JobTemplate,
) -> Result<impl Reply, Infallible> {
    let cronjob = match k8s_client
        .get_cronjob(&job.cronjob_data.namespace, &job.cronjob_data.name)
        .await
    {
        Ok(c) => c,
        Err(e) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"message": e.to_string()})),
                http::StatusCode::BAD_REQUEST,
            ))
        }
    };

    let mut job_spec = cronjob.spec.unwrap().job_template.spec.unwrap();
    let pod_spec = job_spec.template.spec.unwrap();
    job_spec.template.spec = Some(PodSpec {
        containers: merge_job_container(pod_spec.containers, job.spec.containers),
        ..pod_spec
    });
    let new_job = Job {
        metadata: ObjectMeta {
            name: Some(create_tmp_job_name(job.cronjob_data.name.clone())),
            namespace: Some(job.cronjob_data.namespace.clone()),
            ..Default::default()
        },
        spec: Some(job_spec),
        ..Default::default()
    };
    match k8s_client
        .create_job(&job.cronjob_data.name, &new_job)
        .await
    {
        Ok(_) => Ok(warp::reply::with_status(
            warp::reply::json(&json!({
                "message": format!("job '{}' was created", new_job.metadata.name.unwrap())
            })),
            http::StatusCode::CREATED,
        )),
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&json!({"message": e.to_string()})),
            http::StatusCode::BAD_REQUEST,
        )),
    }
}

fn merge_job_container(base: Vec<K8sContainer>, compare: Vec<Container>) -> Vec<K8sContainer> {
    let mut result: Vec<K8sContainer> = base
        .iter()
        .filter_map(|b| {
            let c = match compare.iter().find(|c| c.name == b.name) {
                Some(c) => c,
                None => return None,
            };
            let mut container = b.clone();
            container.image = Some(c.image.clone());
            container.command = match c.command.len() {
                0 => None,
                _ => Some(c.command.clone()),
            };
            container.args = match c.args.len() {
                0 => None,
                _ => Some(c.args.clone()),
            };
            container.env = match c.env.len() {
                0 => None,
                _ => Some(c.env.clone()),
            };
            Some(container)
        })
        .collect();
    result.extend(
        compare
            .iter()
            .filter(|c| base.iter().all(|b| b.name != c.name))
            .map(|b| K8sContainer {
                name: b.name.clone(),
                image: Some(b.image.clone()),
                command: Some(b.command.clone()),
                args: Some(b.args.clone()),
                env: Some(b.env.clone()),
                ..Default::default()
            })
            .collect::<Vec<K8sContainer>>(),
    );
    result
}

fn with_k8s<T: K8sClient + Clone + Send>(
    k8s_client: T,
) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || k8s_client.clone())
}

fn json_body() -> impl Filter<Extract = (JobTemplate,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    let k8s_client = K8sClientImpl(client);
    let list_cronjobs = warp::path!("api" / "cronjob")
        .and(with_k8s(k8s_client.clone()))
        .and(warp::query::<ListCronJobParam>())
        .and_then(list_cronjobs);

    let create_job = warp::post()
        .and(warp::path("api"))
        .and(warp::path("job"))
        .and(with_k8s(k8s_client.clone()))
        .and(json_body())
        .and_then(create_job);

    let index_file = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("./build/index.html"));

    let static_files = warp::path("static").and(warp::fs::dir("./build/static"));

    let other_static_files = warp::fs::dir("./build/");

    let routes = other_static_files
        .or(list_cronjobs)
        .or(create_job)
        .or(index_file)
        .or(static_files);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}
