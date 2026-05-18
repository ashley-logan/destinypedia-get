#[cfg(test)]
mod tests {
    use crossbeam_channel::{Receiver, Sender, unbounded};
    use destinypedia::request::error::RequestResult;
    use destinypedia::sync::cm_request_worker;
    const ROOT_ID: u32 = 364;
    const ROOT_SUBCATS: u16 = 5;
    const ROOT_MEMBERS: [u32; 5] = [363, 369, 31716, 375, 510];

    #[tokio::test]
    async fn test_request_worker() {
        let now = tokio::time::Instant::now();
        let (send, recv) = unbounded::<u32>();
        let (send2, recv2) = unbounded::<(u32, Vec<u8>)>();
        let h = tokio::spawn(cm_request_worker(recv, send2));

        for id in ROOT_MEMBERS {
            send.send(id).unwrap();
        }

        drop(send);
        h.await.unwrap().unwrap();

        while let Ok((id, _bytes)) = recv2.recv() {
            dbg!(format!("fetched page #{}", id));
        }

        let el = now.elapsed();
        dbg!(format!(
            "request worker took {} seconds to run",
            el.as_secs_f64()
        ));
    }

    #[tokio::test]
    async fn test_empty_request_worker() {
        let (send, recv) = unbounded::<u32>();
        let (send2, recv2) = unbounded::<(u32, Vec<u8>)>();
        let h = tokio::spawn(cm_request_worker(recv, send2));

        drop(send);

        h.await.unwrap().unwrap();

        while let Ok((id, _bytes)) = recv2.recv() {
            dbg!(format!("fetched page #{}", id));
        }
    }

    #[tokio::test]
    async fn test_high_traffic_req_worker() {
        let (send, recv) = unbounded::<u32>();
        let (send2, recv2) = unbounded::<(u32, Vec<u8>)>();
        (0..500_u32).for_each(|id| {
            send.send(id).unwrap();
        });
        let mut jset: tokio::task::JoinSet<RequestResult<()>> = tokio::task::JoinSet::new();

        for _ in 0..10 {
            let (recv_copy, send_copy) = (recv.clone(), send2.clone());
            jset.spawn(cm_request_worker(recv_copy, send_copy));
        }

        drop(send);
        drop(recv);
        drop(send2);

        let r = jset.join_all().await;

        while let Ok((id, _bytes)) = recv2.recv() {
            if id % 100 == 0 {
                dbg!(format!("fetched page {}", id));
            }
        }

        assert!(r.iter().all(|r| r.is_ok()))
    }
}
