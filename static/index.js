function init() {
  let screen = document.getElementById('screen');
  let urlbar = document.getElementById('urlbar');

  let url = new URL(window.location);
  url.pathname += 'ws';
  url.protocol = (url.protocol === 'https:') ? 'wss' : 'ws';
  let socket = new WebSocket(url);

  socket.onopen = function (e) {
    console.log('socket opened');
  };

  socket.onmessage = function (event) {
    console.log('msg received ', event.data);
    let url = URL.createObjectURL(event.data);
    document.getElementById('screen').src = url;
  };

  socket.onclose = function (event) {
    console.log('socket closed');
  }

  socket.onerror = function (error) {
    console.log('socket error ', error.message);
  }

  screen.addEventListener('click', function (event) {
    socket.send(
      JSON.stringify({
        type: 'click',
        x: event.offsetX,
        y: event.offsetY,
      })
    );
  });

  urlbar.addEventListener('keydown', event => {
    if (event.key === 'Enter') {
      socket.send(
        JSON.stringify({
          type: 'url',
          url: urlbar.value,
        })
      );
    }
  })
}

document.addEventListener('DOMContentLoaded', init);