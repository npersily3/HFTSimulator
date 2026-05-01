// Bridge server: sits between the Rust exchange and the browser.
//
// What it does:
//   1. Connects to the exchange over TCP and reads "TICK:..." lines.
//   2. Forwards each line to every connected browser over WebSocket.
//   3. Serves public/index.html over HTTP on the same port.
//
// Why a bridge is needed:
//   Browsers can't open raw TCP sockets — they only speak HTTP and WebSocket.
//   The bridge translates the exchange's plain TCP stream into a WebSocket
//   stream that any browser tab can subscribe to.

const http = require('http');
const fs   = require('fs');
const path = require('path');
const net  = require('net');
const { WebSocketServer } = require('ws');

const EXCHANGE_HOST = process.env.EXCHANGE_HOST || 'localhost';
const EXCHANGE_PORT = Number(process.env.EXCHANGE_PORT) || 9000;
const HTTP_PORT     = Number(process.env.HTTP_PORT)     || 8000;

// ── HTTP server ──────────────────────────────────────────────────────────────
// Serves the single HTML page.  The WebSocket server shares this same port
// via the `server` option below, so only one port needs to be exposed.
const server = http.createServer((req, res) => {
    fs.readFile(path.join(__dirname, 'public', 'index.html'), (err, data) => {
        if (err) { res.writeHead(500); res.end('internal error'); return; }
        res.writeHead(200, { 'Content-Type': 'text/html' });
        res.end(data);
    });
});

// ── WebSocket server ─────────────────────────────────────────────────────────
// Attached to the HTTP server so ws:// and http:// share port 8000.
// Each browser tab that opens the page connects here to receive tick data.
const wss      = new WebSocketServer({ server });
const browsers = new Set(); // one entry per open tab

wss.on('connection', ws => {
    browsers.add(ws);
    console.log(`[bridge] browser connected (${browsers.size} open)`);

    ws.on('close', () => {
        browsers.delete(ws);
        console.log(`[bridge] browser disconnected (${browsers.size} remaining)`);
    });
});

// Send a message to every open browser tab
function broadcast(msg) {
    for (const ws of browsers) {
        if (ws.readyState === ws.OPEN) ws.send(msg);
    }
}

// ── Exchange TCP client ──────────────────────────────────────────────────────
// Connects to the exchange, buffers incoming data into complete lines,
// and broadcasts any TICK line to the browsers.
// Retries automatically — the exchange might not be ready immediately on startup.
function connectToExchange() {
    const socket = new net.Socket();
    let   buf    = ''; // partial line carried over between TCP chunks

    socket.connect(EXCHANGE_PORT, EXCHANGE_HOST, () => {
        console.log(`[bridge] connected to exchange at ${EXCHANGE_HOST}:${EXCHANGE_PORT}`);
    });

    socket.on('data', chunk => {
        // TCP delivers data in arbitrary-sized chunks, not guaranteed to align
        // with line boundaries.  We accumulate chunks until we have full lines.
        buf += chunk.toString();
        const lines = buf.split('\n');

        // The last element is either empty (ended on '\n') or an incomplete line.
        // Either way, keep it in the buffer for the next chunk.
        buf = lines.pop();

        for (const line of lines) {
            if (line.startsWith('TICK:')) broadcast(line);
        }
    });

    socket.on('error', err => {
        // Don't crash — the exchange may not be up yet.  The 'close' event
        // fires after 'error', so the retry happens there.
        console.error(`[bridge] exchange error: ${err.message}`);
    });

    socket.on('close', () => {
        console.log('[bridge] exchange connection closed — retrying in 2 s…');
        setTimeout(connectToExchange, 2000);
    });
}

connectToExchange();

server.listen(HTTP_PORT, () => {
    console.log(`[bridge] open http://localhost:${HTTP_PORT} in your browser`);
});
