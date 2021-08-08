FROM node:14

WORKDIR /usr/src/ject-compile
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
CMD ["node", "src/server.js"]