FROM node:24-slim AS nodebt
RUN npm install -g pnpm
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

FROM nodebt AS deps

# Install build tools required by node-gyp for native modules (bufferutil, utf-8-validate, utp-native)
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 \
    make \
    g++ \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /app/workspace
WORKDIR /app
COPY package.json /app/
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --no-frozen-lockfile

FROM deps AS app

COPY main.ts /app/

EXPOSE 6080
EXPOSE 6081
EXPOSE 6082

CMD [ "npm", "start" ]
