use std::env;
use job_hopper::handler::handle;
use job_hopper::k8s::K8sClientImpl;
use kube::Client;
use warp::Filter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "job_hopper=info")
    }
    stackdriver_logger::init_with_cargo!();
    let client = Client::try_default().await?;
    let k8s_client = K8sClientImpl::new(client);
    let api = handle(k8s_client);
    let routes = api.with(warp::log("job_hopper"));
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}
