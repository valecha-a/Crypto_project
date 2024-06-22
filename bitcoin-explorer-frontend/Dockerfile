# Use a Node.js base image with LTS (Long Term Support) version
FROM node:lts as builder

# Set the working directory inside the container
WORKDIR /app

# Copy the package.json and package-lock.json (if available)
COPY package*.json ./

# Install dependencies
RUN npm install

# Copy the entire current directory into the container
COPY . .

# Build the React app
RUN npm run build

# Start a new stage with a lightweight Node.js image
FROM node:lts-alpine

# Set the working directory inside the container
WORKDIR /app

# Copy the built React app from the builder stage to this stage
COPY --from=builder /app/build /app/build

# Expose the port your React app runs on (usually 3000)
EXPOSE 3000

# Command to run your React app when the container starts
CMD ["npx", "serve", "-s", "build"]
