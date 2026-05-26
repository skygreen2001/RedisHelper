FROM node:20-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

EXPOSE 10000
EXPOSE 80/tcp
EXPOSE 443/tcp

CMD ["node", "server/ws-proxy.js"]