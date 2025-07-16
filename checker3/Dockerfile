FROM python:3.13-slim-bookworm
RUN apt-get update && apt-get upgrade -y 

# add checker user
RUN useradd -ms /bin/bash -u 1000 checker

WORKDIR /checker

COPY requirements.txt .

USER checker
RUN python3 -m pip install -r requirements.txt

COPY ./src .

ENTRYPOINT [ "/home/checker/.local/bin/gunicorn", "-c", "gunicorn.conf.py", "checker:app" ]
