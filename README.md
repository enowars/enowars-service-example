n0t3b00k example service
====================
Example service to provide students a project strucutre. n0t3b00k is a simple service that allows users to register, login and save/retrieve notes. 

-----------------------------------------------

# Usage

Use this repository as the base structure for your service. Please keep the directory structure and the following required files:

- `README.md` with a description of your vulnerabilities and exploits.
- `LICENSE` with the MIT
- `.gitignore` files exclude directories or files from being committed.
- `.dockerignore` files exclude directories or files from being tracked by the docker daemon.
- `.env` files used by docker-compose to assign a unique project-name.
- `docker-compose.yml` files to manage your service or checker containers
- `Dockerfile` with commands to build your service

# Required changes

- You'll probably want to through all of these files and replace `n0t3b00k` with your service's name accordingly. 

- Assign your service a unique port. (See `service/docker-compose.yml`, `service/Dockerfile`, `service/src/n0t3b00k.py` and `checker/checker.py`)

# Checking your service

You will have to implement a checker script, which periodically interacts with your service to store and retrieve flags and checks if it still behaves correctly. The game engine will call your checker during a CTF. Use the web interface or `enochecker_cli` to call your different checker methods. 

## Manually
However, you can also perform all game engine call manually in your local development environment. 

- First, start your service with `cd service` and `docker-compose up --build`. 
- Next, start your checker with `cd checker` and `docker-compose up --build`. 

### Web interface
The checker launches a web interface on the port configured in its `docker-compose.yml`.

- Browse to `http://localhost:<checker-port>` to reach the checker interface.

### enochecker_cli

Install `enochecker_cli` using `pip install --user enochecker_cli`. Provide the needed checker URL (`http://localhost:8000`), service IP address (i.e. `192.168.2.112`) and the checker methods to call:

```
$> enochecker_cli -A http://localhost:8000/ -a 192.168.2.112 putflag
OK
$> enochecker_cli -A http://localhost:8000/ -a 192.168.2.112 getflag
OK
$> enochecker_cli -A http://localhost:8000/ -a 192.168.2.112 putnoise
OK
$> enochecker_cli -A http://localhost:8000/ -a 192.168.2.112 getnoise
OK
$> enochecker_cli -A http://localhost:8000/ -a 192.168.2.112 -v 2 havoc
OK
```

## Automatically
You will use CI/CD to continuously check the checker and service. 

- First, install `enochecker_test`.
- Wipe your checker's and service's `./data/` directories as `enochecker_test` requires a clean state.
- Run `enochecker_test`:

```
ENOCHECKER_TEST_CHECKER_ADDRESS='localhost' ENOCHECKER_TEST_CHECKER_PORT='8000' ENOCHECKER_TEST_SERVICE_ADDRESS='192.168.2.112' enochecker_test
```

# Questions?

We understand that this can be a bit overwhelming at first, but you'll quickly get used to the workflow. Nonetheless, *please* reach out to us if you're having problems getting started or something is unclear.