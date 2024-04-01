FROM node:alpine AS build

COPY ./frontend/src/ /app/src/
COPY ./frontend/static/ /app/static/
COPY ["./frontend/package.json", "./frontend/vite.config.ts", "./frontend/tsconfig.json", "./frontend/tailwind.config.ts","./frontend/svelte.config.js","./frontend/postcss.config.cjs","/app/" ]

WORKDIR /app/

RUN npm install
RUN npm run build

FROM node:alpine as production

WORKDIR /app

COPY --from=build /app/build ./build
COPY --from=build /app/node_modules ./node_modules
COPY --from=build /app/package.json .

ENV NODE_ENV=production
ENV PORT=80

EXPOSE 80

ENTRYPOINT [ "node", "build" ]

