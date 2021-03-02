import {inspect} from 'util';
import {createClient} from 'redis';
import {Interaction} from 'slashy';

const sub = createClient();
const client = createClient();

sub.on('subscribe', channel => {
  console.log('Subscribed to channel:', channel);
});

sub.on('message', async (channel, msg) => {
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

    case 'stats': {
      try {
        const stats = await new Promise((resolve, reject) => {
          client.hgetall('events', (err, data) =>
            err ? reject(err) : resolve(data)
          );
        });

        await i.send(
          'have some stats :nerd: ```json\n' + inspect(stats) + '```'
        );
      } catch (err) {
        await i.send(':x: There was an error :|');
      }

      break;
    }
  }
});

sub.subscribe('gateway:INTERACTION_CREATE');
