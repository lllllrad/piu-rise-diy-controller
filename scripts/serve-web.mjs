import { createReadStream } from "node:fs";
import { extname, join, normalize } from "node:path";
import { createServer } from "node:http";
import { fileURLToPath } from "node:url";

const port = Number(process.argv[2] ?? 8000);
if (!Number.isInteger(port) || port < 1 || port > 65535) {
  throw new Error("port must be an integer between 1 and 65535");
}

const root = normalize(fileURLToPath(new URL("../web/", import.meta.url)));
const types = { ".html": "text/html; charset=utf-8", ".css": "text/css; charset=utf-8", ".js": "text/javascript; charset=utf-8" };
const server = createServer((request, response) => {
  const relative = request.url === "/" ? "index.html" : decodeURIComponent(request.url.slice(1));
  const path = normalize(join(root, relative));
  if (!path.startsWith(root)) {
    response.writeHead(403).end("Forbidden");
    return;
  }
  const stream = createReadStream(path);
  stream.on("error", () => response.writeHead(404).end("Not found"));
  response.setHeader("Content-Type", types[extname(path)] ?? "application/octet-stream");
  stream.pipe(response);
});

server.listen(port, "127.0.0.1", () => {
  console.log(`PIU RISE layout preview: http://127.0.0.1:${port}/`);
  console.log("Press Ctrl+C to stop.");
});
