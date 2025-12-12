FROM node:24-alpine AS builder

WORKDIR /app

RUN corepack enable

COPY package.json yarn.lock .yarnrc.yml ./
RUN yarn install --immutable

COPY . .

ARG VITE_API_BASE_URL
ARG VITE_LAUNCHER_NAME
ARG VITE_GITHUB_URL
ENV VITE_API_BASE_URL=${VITE_API_BASE_URL}
ENV VITE_LAUNCHER_NAME=${VITE_LAUNCHER_NAME}
ENV VITE_GITHUB_URL=${VITE_GITHUB_URL}
RUN yarn build-only

FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
