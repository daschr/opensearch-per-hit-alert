<div align="center">
  <h1>wazuh-per-hit-alert</h1>
  <em>Notification services which queries an OpenSearch instance of Wazuh and alerts users using webhooks for each hit.</em><br><br>
  <em>Born out of the frustration that Wazuh is unable to send per-event notifications.</em>
</div>

[![Docker](https://img.shields.io/badge/Docker-2496ED?logo=docker&logoColor=fff)](https://hub.docker.com/r/daschr/wazuh-per-hit-alert) ![docker build](https://github.com/daschr/wazuh-per-hit-alert/actions/workflows/docker-image.yml/badge.svg) 

## Capabilities
- query Wazuh's OpenSearch for for arbitrary events
- notify a user using webhooks (POST) for each hit

## Installation (Docker)
1. use the provided [docker-compose.yml](https://github.com/daschr/wazuh-per-hit-alert/blob/main/docker-compose.yml) and spawn the container
2. got into the `etc` docker-volume of the container and edit the config.toml
3. restart the container and check that it's running
4. That's it! You can now test the service by generating some events which will be returned by you configured queries.
