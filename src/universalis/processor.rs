use anyhow::Result;
use futures::{
    future::{ready, BoxFuture},
    FutureExt, Stream, StreamExt,
};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};
use tokio::time::Instant;

use super::{process_json, MarketBoardInfo, UniversalisRequest};

/////////////////////////////////////////////////////////

#[derive(Clone, PartialEq)]
enum ProcessorStatus {
    Ready,
    Processing,
    Done,
}

impl ProcessorStatus {
    fn is_done(&self) -> bool {
        *self == ProcessorStatus::Done
    }
}

#[derive(Clone)]
pub struct Processor {
    data: Arc<Mutex<ProcessorData>>,
}

struct ProcessorData {
    requests: Vec<UniversalisRequest>,
    status: Vec<ProcessorStatus>,
    all_mb_info: BTreeMap<String, MarketBoardInfo>,
    log_writer: BufWriter<File>,
    start: Instant,
}

type ProcessorReturn = Result<(usize, String)>;
type ProcessorFuture = BoxFuture<'static, ProcessorReturn>;

/////////////////////////////////////////////////////////

impl Processor {
    pub fn new(
        requests: Vec<UniversalisRequest>,
        homeworld: &str,
        data_center: &str,
    ) -> Result<Self> {
        let data = ProcessorData::new(requests, homeworld, data_center)?;

        Ok(Self {
            data: Arc::new(Mutex::new(data)),
        })
    }

    pub async fn process(self) -> Result<BTreeMap<String, MarketBoardInfo>> {
        let mut last_update = Instant::now();
        self.clone()
            .buffer_unordered(8)
            .for_each(|value| {
                let mut stream = self.clone();
                if last_update.elapsed().as_secs() > 10 {
                    stream.update();
                    last_update = Instant::now();
                }
                stream.process_result(value);
                ready(())
            })
            .await;

        match Arc::try_unwrap(self.data) {
            Err(_) => panic!(),
            Ok(v) => match v.into_inner() {
                Err(_) => panic!(),
                Ok(v) => Ok(v.all_mb_info),
            },
        }
    }

    fn process_result(&mut self, result: ProcessorReturn) {
        self.data.lock().unwrap().process_result(result)
    }

    fn update(&self) {
        self.data.lock().unwrap().update()
    }
}

/////////////////////////////////////////////////////////

impl Stream for Processor {
    type Item = ProcessorFuture;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut processor = self.data.lock().unwrap();

        if processor.finished() {
            return Poll::Ready(None);
        }

        async fn send_request(id: usize, url: String) -> ProcessorReturn {
            Ok((id, reqwest::get(&url).await?.text().await?))
        }

        for (id, state) in processor.status.iter().enumerate() {
            match state {
                ProcessorStatus::Ready => {
                    processor.log(format!("[{id:<6}] PROCESSING")).unwrap();

                    processor.status[id] = ProcessorStatus::Processing;
                    let url = processor.requests[id].url.clone();
                    return Poll::Ready(Some(send_request(id, url).boxed()));
                }
                _ => {}
            }
        }

        Poll::Pending
    }
}

/////////////////////////////////////////////////////////

impl ProcessorData {
    fn new(requests: Vec<UniversalisRequest>, homeworld: &str, data_center: &str) -> Result<Self> {
        let status = vec![ProcessorStatus::Ready; requests.len()];
        let log_writer = BufWriter::new(File::create("errors.txt")?);

        let mut all_mb_info = BTreeMap::<String, MarketBoardInfo>::new();
        all_mb_info.insert(homeworld.into(), MarketBoardInfo::new());
        all_mb_info.insert(data_center.into(), MarketBoardInfo::new());

        Ok(Self {
            requests,
            status,
            all_mb_info,
            log_writer,
            start: Instant::now(),
        })
    }

    fn log(&mut self, msg: String) -> Result<()> {
        write!(
            &mut self.log_writer,
            "{:<10.3} {msg}\n",
            self.start.elapsed().as_secs_f32()
        )?;
        self.log_writer.flush()?;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.status
            .iter()
            .all(|status| status == &ProcessorStatus::Done)
    }

    fn process_result(&mut self, result: ProcessorReturn) {
        match result {
            Err(e) => panic!("{:?}", e),
            Ok((id, response)) => {
                match process_json(&self.requests[id], &response, &mut self.all_mb_info) {
                    Err(_e) => {
                        self.log(format!("[{id:<6}] ERROR: {_e:?}")).unwrap();
                        self.status[id] = ProcessorStatus::Ready;
                    }
                    Ok(_) => {
                        self.log(format!("[{id:<6}] DONE")).unwrap();
                        self.status[id] = ProcessorStatus::Done;
                    }
                }
            }
        }
    }

    fn update(&self) {
        let processed = self.status.iter().filter(|status| status.is_done()).count();
        let rate = processed as f32 / self.start.elapsed().as_secs_f32();
        println!(
            "{:<6.1} {processed} processed (ETA {:.1}s)",
            self.start.elapsed().as_secs_f32(),
            self.requests.len() as f32 / rate
        );
    }
}
