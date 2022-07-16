use anyhow::{bail, Result};
use futures::{
    future::{ready, BoxFuture},
    FutureExt, Stream, StreamExt,
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    pin::Pin,
    rc::Rc,
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
pub struct ProcessorStream {
    data: Rc<RefCell<ProcessorData>>,
}

struct ProcessorData {
    requests: Vec<UniversalisRequest>,
    status: Vec<ProcessorStatus>,
    log_writer: BufWriter<File>,
    start: Instant,
}

type ProcessorReturn = (usize, String);
type ProcessorFuture = BoxFuture<'static, ProcessorReturn>;
type ProcessorOutput = BTreeMap<String, MarketBoardInfo>;

/////////////////////////////////////////////////////////

impl ProcessorStream {
    pub fn new(requests: Vec<UniversalisRequest>) -> Result<Self> {
        let data = ProcessorData::new(requests)?;

        Ok(Self {
            // data: Arc::new(Mutex::new(data)),
            data: Rc::new(RefCell::new(data)),
        })
    }

    pub async fn process(
        &self,
        homeworld: &str,
        data_centers: &Vec<&str>,
    ) -> BTreeMap<String, MarketBoardInfo> {
        let mut last_update = Instant::now();

        let mut mb_info = BTreeMap::<String, MarketBoardInfo>::new();
        mb_info.insert(homeworld.into(), MarketBoardInfo::new());
        for &data_center in data_centers {
            mb_info.insert(data_center.into(), MarketBoardInfo::new());
        }

        self.clone()
            .buffer_unordered(8)
            .for_each(|value| {
                if last_update.elapsed().as_secs() > 10 {
                    self.with_inner(|data| data.update());
                    last_update = Instant::now();
                }

                self.with_inner(|data| data.process_result(&value, &mut mb_info));
                ready(())
            })
            .await;

        mb_info
    }
}

/////////////////////////////////////////////////////////

trait Inner {
    type Type;
    fn with_inner<T, F: FnMut(&mut Self::Type) -> T>(&self, func: F) -> T;
    fn try_into_inner(self) -> Result<Self::Type>;
    fn into_inner(self) -> Self::Type
    where
        Self: Sized,
    {
        self.try_into_inner().unwrap()
    }
}

impl Inner for ProcessorStream {
    type Type = ProcessorData;

    fn with_inner<T, F: FnMut(&mut Self::Type) -> T>(&self, mut func: F) -> T {
        let mut data = self.data.borrow_mut();
        func(&mut data)
    }

    fn try_into_inner(self) -> Result<Self::Type> {
        match Rc::try_unwrap(self.data) {
            Err(_) => bail!("Couldn't unwrap rc"),
            Ok(v) => Ok(v.into_inner()),
        }
    }
}

impl Stream for ProcessorStream {
    type Item = ProcessorFuture;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut processor = self.data.borrow_mut();

        if processor.finished() {
            return Poll::Ready(None);
        }

        async fn send_request(id: usize, url: String) -> ProcessorReturn {
            (id, reqwest::get(&url).await.unwrap().text().await.unwrap())
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
    fn new(requests: Vec<UniversalisRequest>) -> Result<Self> {
        let status = vec![ProcessorStatus::Ready; requests.len()];
        let log_writer = BufWriter::new(File::create("errors.txt")?);

        Ok(Self {
            requests,
            status,
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

    fn process_result(&mut self, data: &ProcessorReturn, out_data: &mut ProcessorOutput) {
        let (id, response) = data;
        self.status[*id] = match process_json(&self.requests[*id], &response, out_data) {
            Err(_e) => {
                self.log(format!("[{id:<6}] ERROR: {_e:?}")).unwrap();
                ProcessorStatus::Ready
            }
            Ok(_) => {
                self.log(format!("[{id:<6}] DONE")).unwrap();
                ProcessorStatus::Done
            }
        }
    }

    fn update(&self) {
        let processed = self.status.iter().filter(|status| status.is_done()).count();
        let rate = processed as f32 / self.start.elapsed().as_secs_f32();
        println!(
            "{:<6.1} {processed} processed (ETA {:.1}s, {:.1}/s)",
            self.start.elapsed().as_secs_f32(),
            self.requests.len() as f32 / rate,
            rate,
        );
    }
}
