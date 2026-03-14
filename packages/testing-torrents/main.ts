import fs from "node:fs";
import fsp from "node:fs/promises";
import path from "node:path";
import fastifyStatic from "@fastify/static";
// @ts-expect-error
import TrackerServer from "bittorrent-tracker/server";
import Fastify from "fastify";
import WebTorrent, { type Torrent } from "webtorrent";

// Configuration
const API_PORT = 6080;
const TRACKER_PORT = 6081;
const SEEDING_PORT = 6082;
const STATIC_API_PATH = "/api/static";
const LOCAL_IP = "127.0.0.1";
const WORKSPACE_PATH = "workspace";
const TRACKER_URL = `http://${LOCAL_IP}:${TRACKER_PORT}/announce`;
const API_BASE_URL = `http://${LOCAL_IP}:${API_PORT}${STATIC_API_PATH}/`;

// Initialize Fastify instance
const app = Fastify({ logger: true });

// Mount static file service, mapping ./workspace directory to /api/static route
app.register(fastifyStatic, {
  root: path.join(process.cwd(), WORKSPACE_PATH),
  prefix: STATIC_API_PATH,
});

const tracker = new TrackerServer({
  udp: false, // enable udp server? [default=true]
  http: true, // enable http server? [default=true]
  ws: false, // enable websocket server? [default=true]
  stats: true, // enable web-based statistics? [default=true]
  trustProxy: true, // enable trusting x-forwarded-for header for remote IP [default=false]
});

// Define request and response type definitions
interface FileItem {
  path: string;
  size: number;
}

interface RequestSchema {
  id: string;
  fileList: FileItem[];
}

interface ResponseSchema {
  torrentUrl: string;
  magnetUrl: string;
  hash: string;
}

// Start local Tracker
async function startTracker(): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    tracker.listen(TRACKER_PORT, "0.0.0.0", () => {
      console.log(`Tracker listening on port ${TRACKER_PORT}`);
      resolve();
    });
    tracker.on("error", (err: any) => {
      reject(`Tracker error: ${err}`);
    });
    tracker.on("warning", (warn: any) =>
      console.warn(`Tracker warning: ${warn}`),
    );
    // Log tracked torrents
    tracker.on("update", (addr: any, params: any) => {
      console.log(`Tracker update: ${params.info_hash} from ${addr}`);
    });
  });
}

// Tracker and WebTorrent client
const webTorrent = new WebTorrent({
  // @ts-expect-error
  torrentPort: SEEDING_PORT,
});

// Generate mock file
async function generateMockFile(filePath: string, size: number) {
  const dir = path.dirname(filePath);
  if (!fs.existsSync(dir)) {
    await fsp.mkdir(dir, { recursive: true });
  }

  await fsp.writeFile(filePath, Buffer.alloc(0));
  await fsp.truncate(filePath, size);
}

// Add bittorrent and seed
async function seedTorrent(
  torrentPath: string,
  contentFolder: string,
): Promise<Torrent> {
  return new Promise((resolve) => {
    const torrent = webTorrent.seed(
      contentFolder,
      {
        announceList: [[TRACKER_URL]], // Specify tracker URL
        private: false,
        createdBy: "Konobangu Testing Torrents",
        urlList: [API_BASE_URL],
      },
      async (t) => {
        await fsp.writeFile(torrentPath, t.torrentFile);
        resolve(t);
      },
    );
    torrent.on("error", (err) => console.error(`Torrent error: ${err}`));
    torrent.on("wire", (wire) =>
      console.log(`Connected to peer: ${wire.peerId}`),
    );
    torrent.on("done", () =>
      console.log(`Torrent ${torrent.infoHash} fully seeded`),
    );
  });
}

// Handle POST request to /api/torrents/mock
app.post<{ Body: RequestSchema }>("/api/torrents/mock", async (req, _reply) => {
  const { id, fileList } = req.body;

  const idFolder = path.join(WORKSPACE_PATH, id);
  if (!fs.existsSync(idFolder)) {
    await fsp.mkdir(idFolder, { recursive: true });
  }

  for (const fileItem of fileList) {
    const filePath = path.join(idFolder, fileItem.path);
    await generateMockFile(filePath, fileItem.size);
  }

  const torrentPath = path.join(WORKSPACE_PATH, `${id}.torrent`);

  const torrent = await seedTorrent(torrentPath, idFolder);
  const magnetUrl = `magnet:?xt=urn:btih:${torrent.infoHash}&tr=${TRACKER_URL}`;

  return {
    torrentUrl: `${API_BASE_URL}${id}.torrent`,
    magnetUrl,
    hash: torrent.infoHash,
  } as ResponseSchema;
});

// Main program entry
async function main() {
  try {
    await startTracker();
    const address = await app.listen({ port: API_PORT, host: "0.0.0.0" });
    console.log("Listening on:", address);
  } catch (err) {
    console.error("Startup error:", err);
    webTorrent.destroy();
    tracker.close();
    process.exit(1);
  }
}

main();

// Graceful shutdown
process.on("SIGINT", () => {
  console.log("Shutting down...");
  tracker.close();
  webTorrent.destroy();
  process.exit(0);
});
