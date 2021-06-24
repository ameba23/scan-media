const Net = require('net')
const port = 29394
const host = 'localhost'

const client = new Net.Socket()
client.connect({ port, host }, () => {
  console.log('TCP connection established with the server.');
  client.write('Hello, server.')
}).on('error', console.log)

client.on('data', function(chunk) {
    console.log(`Data received from the server: ${chunk.toString()}.`);
    client.end();
});

client.on('end', function() {
    console.log('Requested an end to the TCP connection');
})
