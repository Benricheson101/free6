import {createClient} from 'redis';
import {Interaction} from 'slashy';

const sub = createClient();

sub.on('message', async (_channel, msg) => {
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

sub.subscribe('gateway:INTERACTION_CREATE');
