FROM node:22-alpine
WORKDIR /app
COPY package.json ./
RUN npm install --omit=dev
COPY src ./src
COPY public ./public
ENV PORT=8096
EXPOSE 8096
CMD ["npm", "start"]
