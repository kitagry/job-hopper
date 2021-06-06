use job_hopper::handler::handle;
use job_hopper::k8s::K8sClientImpl;
use kube::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    let k8s_client = K8sClientImpl::new(client);
    let routes = handle(k8s_client);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}
