#[cfg(test)]
mod tests {
    use crossbeam_channel::{Receiver, Sender, unbounded};
    use destiny_fetch::sync::cm_request_worker;
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
        h.await.unwrap();

        for (id, _bytes) in recv2.recv().iter() {
            eprintln!("fetched page {}", id);
        }


        let el = now.elapsed();
        eprintln!(
            "request worker took {} seconds to run",
            el.as_secs_f64()
        );
    }
}
