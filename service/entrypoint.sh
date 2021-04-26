#!/bin/sh
set -e
set -x

# Chown the mounted data volume
chown -R service:service "/data/"

# Launch our service as user 'service'
exec su -s /bin/sh -c 'PYTHONUNBUFFERED=1 python3 n0t3b00k.py' service