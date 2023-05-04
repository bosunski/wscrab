const { WebSocketServer } = require('ws');

const port = 8080;

const eventMap = {
  authenticate: (params) => {}
}

const wss = new WebSocketServer({ port }, () => {
    console.log(`Listening on port ${port}...`)
});

wss.on('connection', function connection(ws, r) {
  console.log(r);
  ws.on('error', console.error);

  ws.on('message', (something) =>  {
    console.log('Received: %s', something);
    ws.send(`You said: ${something}`);
  });

  ws.send('Connected!');
});
