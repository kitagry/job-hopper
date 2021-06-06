use k8s_openapi::api::batch::v1beta1::CronJob;
use k8s_openapi::api::core::v1::EnvVar;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct JobTemplate {
    pub cronjob_data: CronJobData,
    pub spec: JobSpec,
}

#[derive(Deserialize, Serialize)]
pub struct CronJobData {
    pub name: String,
    pub namespace: String,
}

#[derive(Deserialize, Serialize)]
pub struct JobSpec {
    pub containers: Vec<Container>,
}

#[derive(Deserialize, Serialize)]
pub struct Container {
    pub name: String,
    pub image: String,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub env: Vec<EnvVar>,
}

impl JobTemplate {
    pub fn new(cj: CronJob) -> Self {
        JobTemplate {
            cronjob_data: CronJobData {
                name: cj.metadata.name.unwrap_or_else(|| "".to_string()),
                namespace: cj.metadata.namespace.unwrap_or_else(|| "".to_string()),
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
    pub fn new(c: k8s_openapi::api::core::v1::Container) -> Self {
        Container {
            name: c.name,
            image: c.image.unwrap_or_else(|| "".to_string()),
            command: c.command.unwrap_or_default(),
            args: c.args.unwrap_or_default(),
            env: c.env.unwrap_or_default(),
        }
    }
}
