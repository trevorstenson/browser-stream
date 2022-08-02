use actix::{Actor, StreamHandler, Addr, Message, Handler, AsyncContext};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::Deserialize;

use crate::browser::{BrowserManager, Subscribe};

pub struct WebSocketConnection {
  manager: Addr<BrowserManager>
}

impl Actor for WebSocketConnection {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    self.manager.do_send(Subscribe(ctx.address()));
  }
}

#[derive(Message, Deserialize, Debug)]
#[rtype("()")]
#[serde(tag = "type")]
pub enum ClientAction {
  #[serde(rename = "click")]
  Click { x: f64, y: f64 },
  #[serde(rename = "url")]
  URL { url: String },
}

#[derive(Message)]
#[rtype("()")]
pub struct SendScreenData(pub Vec<u8>);

impl Handler<SendScreenData> for WebSocketConnection {
  type Result = ();

  fn handle(&mut self, _msg: SendScreenData, ctx: &mut Self::Context) {
    println!("Sending screen data");
    ctx.binary(_msg.0);
  }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketConnection {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match msg {
      Ok(ws::Message::Text(text)) => {
        println!("Received text: {}", text);
        let action: ClientAction = serde_json::from_str(&text).unwrap();
        println!("Received action: {:?}", action);
        self.manager.do_send(action);
      }
      _ => (),
    }
  }
}

pub async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
  let resp = ws::start(
    WebSocketConnection {
      manager: req.app_data::<Addr<BrowserManager>>().unwrap().clone(),
    },
    &req,
    stream
  );
  println!("my ws resp {:?}", resp);
  resp
}