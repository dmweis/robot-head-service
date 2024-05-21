# Rust service robot-head-service project with compilation for raspberry pi

[![codecov](https://codecov.io/gh/dmweis/robot-head-service/branch/main/graph/badge.svg)](https://codecov.io/gh/dmweis/robot-head-service)
[![Rust](https://github.com/dmweis/robot-head-service/workflows/Rust/badge.svg)](https://github.com/dmweis/robot-head-service/actions)
[![Private docs](https://github.com/dmweis/robot-head-service/workflows/Deploy%20Docs%20to%20GitHub%20Pages/badge.svg)](https://davidweis.dev/robot-head-service/robot-head-service/index.html)

## Commands

```shell
z_put --key robot-head/command --value "{\"active\": false}"
z_put --key robot-head/command --value "{\"active\": true}"
z_put --key robot-head/command --value "{\"active\": true, \"yaw\": 65.0}"
z_put --key robot-head/command --value "{\"yaw\": 65.0}"
```
