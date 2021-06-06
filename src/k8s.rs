use async_trait::async_trait;
use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::api::batch::v1beta1::CronJob;
use kube::{
    api::{Api, ListParams, ObjectList, PostParams},
    Client,
};

#[async_trait]
pub trait K8sClient {
    async fn list_cronjobs(&self, namespace: &str) -> Result<ObjectList<CronJob>, kube::Error>;
    async fn get_cronjob(&self, namespace: &str, name: &str) -> Result<CronJob, kube::Error>;
    async fn create_job(&self, namespace: &str, job: &Job) -> Result<Job, kube::Error>;
}

#[derive(Clone)]
pub struct K8sClientImpl(Client);

impl K8sClientImpl {
    pub fn new(client: Client) -> Self {
        K8sClientImpl(client)
    }
}

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
