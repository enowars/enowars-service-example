# Use a suitable base-image.
FROM python:3-alpine

# Create a seperate user and chown new directories if necesary
RUN addgroup --system service
RUN adduser --system --ingroup service --uid 1000 service
RUN mkdir /data/
RUN chown service:service /data/

# Run the following commands as 'service'
USER service
# Change the working directory.
WORKDIR /service/
# Expose our service's port
EXPOSE 8000

# Run our service
CMD PYTHONUNBUFFERED=1 python3 n0t3b00k.py
