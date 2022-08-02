use std::{
  sync::Arc,
  time::Duration,
};
use actix::{Actor, Addr, AsyncContext, Handler, Message};
use headless_chrome::{
  protocol::{page::ScreenshotFormat, browser::Bounds},
  Browser, Tab,
};

use crate::websocket::{WebSocketConnection, SendScreenData, ClientAction};

const INTERVAL: Duration = Duration::from_millis(20);

#[derive(Message)]
#[rtype("()")]
struct Beat;

#[derive(Message)]
#[rtype("()")]
pub struct Subscribe(pub Addr<WebSocketConnection>);

pub struct BrowserManager {
  clients: Vec<Addr<WebSocketConnection>>,
  pub browser: Browser,
  tab: Arc<Tab>,
  screen_data: Vec<u8>,
  url: String,
}

impl BrowserManager {
  pub fn new(url: &str) -> Self {
    let browser = Browser::default().expect("Couldnt create browser");
    let tab = browser.wait_for_initial_tab().unwrap();
    tab.set_bounds(Bounds::Normal {
      left: Some(0),
      top: Some(0),
      width: Some(1920),
      height: Some(1080)
    }).unwrap();
    tab.navigate_to(url).unwrap();
    tab.wait_until_navigated().unwrap();
    let screen_data = tab.capture_screenshot(ScreenshotFormat::PNG, None, true).unwrap();
    BrowserManager {
      clients: vec![],
      browser,
      tab,
      screen_data: screen_data,
      url: url.to_string(),
    }
  }

  fn process_changes(&mut self) {
    let screen_data = self.tab.capture_screenshot(ScreenshotFormat::PNG, None, true).unwrap();
    if screen_data != self.screen_data {
      println!("Screen data changed, sending to clients");
      self.screen_data = screen_data;
      for client in self.clients.iter() {
        client.do_send(SendScreenData(self.screen_data.clone()));
      }
    }
  }
}

impl Actor for BrowserManager {
  type Context = actix::Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    ctx.notify_later(Beat, INTERVAL);
  }
}

impl Handler<Beat> for BrowserManager {
  type Result = ();

  fn handle(&mut self, _msg: Beat, ctx: &mut Self::Context) {
    self.process_changes();
    ctx.notify_later(Beat, INTERVAL);
  }
}

impl Handler<Subscribe> for BrowserManager {
  type Result = ();

  fn handle(&mut self, msg: Subscribe, _ctx: &mut Self::Context) {
    println!("Subscribing client {:?}", msg.0);
    self.clients.push(msg.0);
  }
}

impl Handler<ClientAction> for BrowserManager {
  type Result = ();

  fn handle(&mut self, msg: ClientAction, ctx: &mut Self::Context) -> Self::Result {
    match msg {
      ClientAction::URL { url } => {
        println!("Changing url to {}", url);
        self.tab.navigate_to(url.as_str()).unwrap();
      },
      ClientAction::Click { x, y } => {
        println!("Clicking at {}, {}", x, y);
        let body = self.tab.wait_for_element("html").unwrap();
        let mut calc_point = body.get_midpoint().unwrap();
        let x_diff = calc_point.x - x;
        let y_diff = calc_point.y - y;
        calc_point.x -= x_diff;
        calc_point.y -= y_diff;
        self.tab.click_point(calc_point).unwrap();
      }
      _ => (),
    }
  }
}