FROM node:20-alpine

WORKDIR /app

# Ставим зависимости
COPY ./frontend/package.json ./frontend/package-lock.json ./
RUN npm ci

# Копируем проект
COPY ./frontend .

# Vite dev server использует порт 5173
EXPOSE 5173

CMD ["npm", "run", "dev", "--", "--host"]
