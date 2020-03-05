#[macro_use]
extern crate quick_error;
extern crate futures;
extern crate rand;

use actix::prelude::*;
use futures::{Future, FutureExt, TryFuture, TryFutureExt};

use rand::Rng;
use std::collections::HashMap;
use std::io;
use std::time::SystemTime;

quick_error! {
    #[derive(Debug)]
    pub enum MyError {
        Io(err: io::Error) {
            from()
            display("I/O error: {}", err)
            cause(err)
        }
        Other(descr: &'static str) {
            display("Error {}", descr)
        }
        IoAt { place: &'static str, err: io::Error } {
            cause(err)
            display(me) -> ("io error at {}: {}", place, err)
            from(s: String) -> {
                place: "some string",
                err: io::Error::new(io::ErrorKind::Other, s)
            }
        }
        Discard {
            from(&'static str)
        }
    }
}

struct HttpHandler {}

impl HttpHandler {
    fn new() -> Self {
        HttpHandler {}
    }
}

struct Filter {}

impl Filter {
    fn new() -> Self {
        Filter {}
    }
}

struct Storage {
    store: HashMap<u32, (String, SystemTime)>,
}
impl Storage {
    fn new() -> Self {
        let store = HashMap::<u32, (String, SystemTime)>::new();
        Storage { store }
    }
}

struct ResponseCreator {}

impl ResponseCreator {
    fn new() -> Self {
        ResponseCreator {}
    }
}

impl Actor for HttpHandler {
    type Context = Context<Self>;
}

impl Actor for Filter {
    type Context = Context<Self>;
}

impl Actor for Storage {
    type Context = Context<Self>;
}

impl Actor for ResponseCreator {
    type Context = Context<Self>;
}

#[derive(MessageResponse, Debug)]
enum Msg {
    Request(String),
    InternalRequest((String, SystemTime)),
    IndexedMsg((String, SystemTime, u32)),
    ResponseIndex(u32),
    ResponseOut((u32, SystemTime)),
    Error(MyError),
}

impl Message for Msg {
    type Result = Msg;
}

impl Handler<Msg> for HttpHandler {
    type Result = Msg;
    fn handle(&mut self, msg: Msg, _ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            Msg::Request(r) => Msg::InternalRequest((r, SystemTime::now())),
            _ => unimplemented!(),
        }
    }
}

impl Handler<Msg> for Filter {
    type Result = Msg;
    fn handle(&mut self, input: Msg, _ctx: &mut Context<Self>) -> Self::Result {
        let mut rng = rand::thread_rng();
        match input {
            Msg::InternalRequest((msg, t)) => Msg::IndexedMsg((msg, t, rng.gen::<u32>())),
            _ => Msg::Error(MyError::Other("wrong input type")),
        }
    }
}

impl Handler<Msg> for Storage {
    type Result = Msg;

    fn handle(&mut self, input: Msg, _ctx: &mut Context<Self>) -> Self::Result {
        match input {
            Msg::IndexedMsg((msg, t, idx)) => {
                self.store.insert(idx, (msg, t));
                Msg::ResponseIndex(idx)
            }
            _ => Msg::Error(MyError::Other("wrong input type")),
        }
    }
}

impl Handler<Msg> for ResponseCreator {
    type Result = Msg;

    fn handle(&mut self, input: Msg, _ctx: &mut Context<Self>) -> Self::Result {
        match input {
            Msg::ResponseIndex(idx) => Msg::ResponseOut((idx, SystemTime::now())),
            _ => Msg::Error(MyError::Other("wrong input type")),
        }
    }
}

fn main() {
    let sys = System::new("example");

    let http = HttpHandler::new().start();
    let filter = Filter::new().start();
    let store = Storage::new().start();
    let resp = ResponseCreator::new().start();

    let exec = http
        .send(Msg::Request("haha".to_string()))
        .and_then(move |r| {
            eprintln!("{:#?}", r);
            filter.send(r)
        })
        .and_then(move |r| {
            eprintln!("{:#?}", r);
            store.send(r)
        })
        .and_then(move |r| {
            eprintln!("{:#?}", r);
            resp.send(r)
        })
        .map(move |e| eprintln!("Done\n{:#?}", e));

    Arbiter::spawn(exec);

    sys.run();
}
