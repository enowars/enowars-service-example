FROM python:3.7

WORKDIR /checker

# Install all required dependencies for the checker.
COPY ./requirements.txt /checker/requirements.txt
RUN pip3 install -r requirements.txt

# Copy all files into the container.
COPY checker.py gunicorn.conf.py ./


ENTRYPOINT [ "gunicorn", "-c", "gunicorn.conf.py", "checker:app" ]