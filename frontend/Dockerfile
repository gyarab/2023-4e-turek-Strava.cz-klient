FROM node:alpine AS build

COPY ./src/ /app/src/
COPY ./static/ /app/static/
COPY ["./package.json", "./vite.config.ts", "tsconfig.json", "tailwind.config.ts","svelte.config.js","postcss.config.cjs","/app/" ] 

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

