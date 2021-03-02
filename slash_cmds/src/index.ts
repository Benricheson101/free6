import {createClient} from 'redis';
import {Interaction} from 'slashy';

const client = createClient();

client.on('subscribe', channel => {
  console.log('Subscribed to channel:', channel);
});

client.on('message', async (channel, msg) => {
  if (channel !== 'gateway:INTERACTION_CREATE') {
    return;
  }
  const i = new Interaction(JSON.parse(msg), '641392996025106432');

  switch (i.data.name) {
    case 'ping': {
      const before = Date.now();
      await i.send(':ping_pong: Pong!');
      const after = Date.now();

      await i.edit(`:ping_pong: Pong! Message sent in ${after - before} ms`);
      break;
    }
  }
});

client.subscribe('gateway:INTERACTION_CREATE');
