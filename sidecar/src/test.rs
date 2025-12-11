pub use crate::*;

#[tokio::test]
async fn processes_spawn_and_shutdown_1k() {
    use rand::random_range;
    use futures::future::join_all;
    use itertools::Itertools;

    let mut tasks = vec![];

    let mut builder = process::ServerProcess::spawner("rcssserver").await;

    for mut ports in (6000..=9000).chunks(3).into_iter() {
        if let Some((server, coach, sidecar)) = ports.next_tuple() {
            builder.config.with_ports(server, coach, sidecar);
        } else { break }

        let mut process = builder.spawn().await.unwrap();
        println!("Process running, pid = {:?}", process.pid());
        let task = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(random_range(5000..10000))).await;
            let ret = process.shutdown().await.unwrap();
            println!("Process terminated, ret code = {ret}")
        });

        tasks.push(task);
    }

    join_all(tasks).await;
}