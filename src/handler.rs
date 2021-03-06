use super::k8s::K8sClient;
use super::model::{Container, JobTemplate};
use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::api::core::v1::{Container as K8sContainer, PodSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::convert::Infallible;
use warp::{Filter, Reply};

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
        .create_job(&job.cronjob_data.namespace, &new_job)
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

pub fn handle<T: K8sClient + Clone + Send + Sync>(
    k8s_client: T,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let list_cronjobs = warp::path!("api" / "cronjob")
        .and(with_k8s(k8s_client.clone()))
        .and(warp::query::<ListCronJobParam>())
        .and_then(list_cronjobs);

    let create_job = warp::post()
        .and(warp::path("api"))
        .and(warp::path("job"))
        .and(with_k8s(k8s_client))
        .and(json_body())
        .and_then(create_job);

    let index_file = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("./build/index.html"));

    let static_files = warp::path("static").and(warp::fs::dir("./build/static"));

    let other_static_files = warp::fs::dir("./build/");

    other_static_files
        .or(list_cronjobs)
        .or(create_job)
        .or(index_file)
        .or(static_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use k8s_openapi::api::batch::v1::{Job, JobSpec};
    use k8s_openapi::api::batch::v1beta1::{CronJob, CronJobSpec, JobTemplateSpec};
    use k8s_openapi::api::core::v1::{Container, PodSpec, PodTemplateSpec};
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use serde_json::json;

    fn new_cronjob() -> CronJob {
        CronJob {
            metadata: ObjectMeta {
                name: Some("test_cron_job".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: Some(CronJobSpec {
                job_template: JobTemplateSpec {
                    spec: Some(JobSpec {
                        template: PodTemplateSpec {
                            spec: Some(PodSpec {
                                containers: vec![Container {
                                    name: "test".to_string(),
                                    image: Some("test".to_string()),
                                    command: Some(vec!["echo".to_string()]),
                                    args: Some(vec!["hello world".to_string()]),
                                    ..Default::default()
                                }],
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[derive(Clone)]
    struct K8sClientMock {
        list_cronjobs: Result<Vec<CronJob>, String>,
    }

    #[async_trait]
    impl K8sClient for K8sClientMock {
        async fn list_cronjobs(&self, _namespace: &str) -> Result<Vec<CronJob>, kube::Error> {
            match self.list_cronjobs.clone() {
                Ok(o) => Ok(o),
                Err(e) => Err(kube::Error::RequestValidation(e)),
            }
        }

        async fn get_cronjob(
            &self,
            _namespace: &str,
            _nameme: &str,
        ) -> Result<CronJob, kube::Error> {
            Ok(new_cronjob())
        }

        async fn create_job(&self, _namespace: &str, _job: &Job) -> Result<Job, kube::Error> {
            Ok(Job {
                ..Default::default()
            })
        }
    }

    #[tokio::test]
    async fn test_list_cronjobs() {
        let routes = handle(K8sClientMock {
            list_cronjobs: Ok(vec![new_cronjob()]),
        });

        let res = warp::test::request()
            .path("/api/cronjob")
            .reply(&routes)
            .await;
        assert_eq!(res.status(), 200);

        let expect = json!([{
            "cronjob_data": {
                "name": "test_cron_job",
                "namespace": "default"
            },
            "spec": {
                "containers": [
                    {
                        "name": "test",
                        "image": "test",
                        "command": ["echo"],
                        "args": ["hello world"],
                        "env": []
                    }
                ]
            }
        }]);
        let res_body: serde_json::Value = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(res_body, expect);
    }

    #[tokio::test]
    async fn test_list_cronjobs_fail() {
        let routes = handle(K8sClientMock {
            list_cronjobs: Err("failed to list cronjob".to_string()),
        });

        let res = warp::test::request()
            .path("/api/cronjob")
            .reply(&routes)
            .await;
        assert_eq!(res.status(), 500);

        let expect = json!({
            "message": "Request validation failed with failed to list cronjob"
        });
        let res_body: serde_json::Value = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(res_body, expect);
    }

    #[tokio::test]
    async fn test_create_job() {
        let routes = handle(K8sClientMock {
            list_cronjobs: Err("not found".to_string()),
        });
        let request = json!({
            "cronjob_data": {
                "name": "test_cron_job",
                "namespace": "default"
            },
            "spec": {
                "containers": [
                    {
                        "name": "test",
                        "image": "test",
                        "command": ["echo"],
                        "args": ["hello world"],
                        "env": []
                    }
                ]
            }
        });
        let res = warp::test::request()
            .path("/api/job")
            .method("POST")
            .body(request.to_string().as_bytes())
            .header("Content-Type", "application/json")
            .reply(&routes)
            .await;
        assert_eq!(res.status(), 201);

        let res_body: serde_json::Value = serde_json::from_slice(res.body()).unwrap();
        assert!(res_body.as_object().unwrap().contains_key("message"));
    }
}
