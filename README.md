# jsonkv-server
Web(socket) server that stores and listens to JSON data. It was specifically created for use in some broadcasts.

# **Warning**
Those code is just of prototype, not meant to be used in production. Those code should be rewritten and tested before being used in production.

## Routes
Those routes require a secret key to be passed in the `Authorization` header.
- `GET, POST, PUT, PATCH /data/[key]`: This route allows you to perform operations on a specific data key. You can retrive via GET, create via POST, update(reset) via PUT, and patch(modify specific object using json-patch) via PATCH.
- `/listen/[key]`: By accessing this route, you can listen to a websocket for changes in a specific data key. You will receive data from the websocket whenever there are changes.
- `GET /list`: Use this route to get a list of all the available keys. Enabled by default, but can be disabled via the config.

## Rules
- All keys must be in English and cannot contain dashes ( - ), underscores ( _ ), or numbers.

## Config
All configuration settings can be set either via a dotenv file or as environment variables.
- `JSONKV_LISTEN`: This determines the address and port the server will listen on. The default setting is `127.0.0.1:19720`
- `JSONKV_DATA_DIR`: This determines the location where the data is stored. The default location is `./data/`.
- `JSONKV_SECRET_FILE`: By setting this variable, you can specify the location of the file where all the secrets are stored. If the file does not exist, it will be created. The default file name is `secret.toml`.
- `JSONKV_ENABLE_LIST`: Enables or disables the data list route. The default setting is `true`.

## TODOs
- [ ] Default data introduction in case of missing data
- [ ] Graceful shutdown
- [ ] Better error handling
- [ ] Better logging

## License
Apache-2.0