# Konobangu Testing Torrents Container

## Development

```bash
pnpm install --ignore-workspace
```

## Build

```bash
docker buildx build --platform linux/amd64 --tag konobangu-testing-torrents:latest --load .
```

## Run

```bash
docker run -p 6080:6080 -p 6081:6081 -p 6082:6082 --name konobangu-testing-torrents  konobangu-testing-torrents:latest
```

## Publish

```bash
docker tag konobangu-testing-torrents:latest ghcr.io/apeiraco/konobangu-testing-torrents:latest
docker push ghcr.io/apeiraco/konobangu-testing-torrents:latest
```
