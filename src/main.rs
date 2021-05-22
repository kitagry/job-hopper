use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::batch::v1::Job;
use kube::{
    api::{Api, DeleteParams, ListParams, PostParams, WatchEvent},
    Client,
};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    let namespace = std::env::var("NAMESPACE").unwrap_or("default".into());

    let job_name = "empty-job";
    let my_job = serde_json::from_value(json!({
        "apiVersion": "batch/v1",
            "kind": "Job",
            "metadata": {
                "name": job_name
            },
        "spec": {
            "template": {
                "metadata": {
                    "name": "pod"
                },
                "spec":{
                    "containers": [{
                        "name": "empty",
                        "image": "alpine:latest"
                    }],
                    "restartPolicy": "Never"
                }
            }
        }
    }))?;

    let jobs: Api<Job> = Api::namespaced(client, &namespace);
    let pp = PostParams::default();

    jobs.create(&pp, &my_job).await?;

    let lp = ListParams::default()
        .fields(&format!("metadata.name={}", job_name))
        .timeout(20);
    let mut stream = jobs.watch(&lp, "").await?.boxed();

    while let Some(status) = stream.try_next().await? {
        match status {
            WatchEvent::Added(s) => println!("added {}", s.metadata.name.unwrap_or("".to_string())),
            WatchEvent::Modified(s) => {
                let name = s.metadata.name.unwrap_or("".to_string());
                let current_status = s.status.clone().expect("status is missing");
                match current_status.completion_time {
                    Some(_) => {
                        println!("Modified: {} is complete", name);
                        break;
                    }
                    _ => println!("Modified: {} is running", name),
                }
            }
            WatchEvent::Deleted(s) => {
                println!("Deleted {}", s.metadata.name.unwrap_or("".to_string()))
            }
            WatchEvent::Error(s) => eprintln!("{}", s),
            _ => {}
        }
    }

    println!("delete the job");
    let mut dp = DeleteParams::default();
    dp.dry_run = true;
    jobs.delete(job_name, &dp).await?;
    dp.dry_run = false;
    jobs.delete(job_name, &dp).await?;

    Ok(())
}
