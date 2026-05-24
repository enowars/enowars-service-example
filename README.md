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

# Troubleshoot

<details>

<summary><b>Checker is not able to communicate with service</b></summary>

**Note 1: The following configuration MUST be removed before the test runs and test-/ final-ctf.**

**Note 2: Before applying the following configuration, check your firewall's INPUT rules.**

If you are trying to reach the service from your checker, normally it should be possible by providing the ip address of your hostmachine (run `ip a` to find it): `enochecker_cli -A http://localhost:<CHECKER_PORT>/ -a <HOST_IP> putflag`. However on some machines this was not working as intended.

Enabling all containers to communicate with each other is possible by creating a docker network:

1. First, define a network within `service/docker-compose.yml` like so:

```yaml
service:
  my-service-container: # this name will also be the DNS entry within the network
    # ...
    networks:
      - my-network

networks:
  my-network:
    driver: bridge
```

2. Start your service's compose without detach (no `-d`) and watch the stdout. It should be printed that the network was created. Copy the name of the network, it should be something like `service_my-network`.

Reference: [Networking in Compose](https://docs.docker.com/compose/networking/)

> Note
> Your app's network is given a name based on the "project name", which is based on the name of the directory it lives in. You can override the project name with either the --project-name flag or the COMPOSE\_PROJECT\_NAME environment variable.

3. Now, add the checker containers to the network within `checker/docker-compose.yml` like so:

```yaml
services:
  my-service-checker:
    # ...
    networks:
      - service_my-network
  my-service-mongo:
    # ...
    networks:
      - service_my-network

networks:
  service_my-network:
    external: True 
```

4. Start your checkers compose.
5. To test, you can attach to the checker's container: Find the name of the checker container by running `docker ps -a`. Now run `docker exec -it <CONTAINER_NAME> bash` and try to send a request to your service container: `curl http://my-service-container:<PORT>` (only works if you have an HTTP API exposed). If the `curl` call returns something, everything should be working now, congrats!
6. From your host you can now use enochecker\_cli to run specific checker tasks like so: `enochecker_cli -A http://localhost:<CHECKER_PORT>/ -a my-service-container <TASK>`

</details>
