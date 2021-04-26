# Use a suitable base-image.
FROM python:3-alpine

# Create a seperate user and chown new directories if necessary
RUN addgroup --system service
RUN adduser --system --ingroup service --uid 1000 service

# Create our mapped data volume endpoint
RUN mkdir /data/

# Copy our entrypoint.sh and make it executable
COPY entrypoint.sh /
RUN chmod +x /entrypoint.sh

# Copy our service
COPY src/ /service/

# Change the working directory.
WORKDIR /service/
# Expose the service's port
EXPOSE 8000

# Run the service
ENTRYPOINT ["/entrypoint.sh"]
