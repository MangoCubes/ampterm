use crate::queryworker::query::ToQueryWorker;

pub struct Delayer {
    queries: Vec<(ToQueryWorker, usize)>,
}

impl Delayer {
    pub fn new() -> Self {
        Self { queries: vec![] }
    }
    pub fn queue_query(&mut self, query: ToQueryWorker, delay: usize) -> Option<ToQueryWorker> {
        let ret = if let Some(pos) = self.queries.iter().position(|(in_queue, _)| {
            std::mem::discriminant(&query.query) == std::mem::discriminant(&in_queue.query)
        }) {
            Some(self.queries.remove(pos).0)
        } else {
            None
        };
        self.queries.push((query, delay));
        ret
    }

    fn countdown(&mut self) {
        self.queries.iter_mut().for_each(|(_, c)| {
            *c -= 1;
        });
    }

    pub fn on_tick(&mut self) -> Option<ToQueryWorker> {
        if let Some(pos) = self.queries.iter().position(|(_, tick)| *tick == 0) {
            let exec = self.queries.remove(pos).0;
            Some(exec)
        } else {
            self.countdown();
            None
        }
    }
}
