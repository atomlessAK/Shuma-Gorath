# Scripts

Automation scripts used by the Makefile live here.

## Layout

- `bootstrap/`
  - `setup.sh`: dependency/bootstrap workflow used by `make setup`
  - `verify-setup.sh`: setup verification used by `make verify`
- `tests/`
  - `integration.sh`: HTTP integration scenarios used by `make test` and `make test-integration`
- `config_seed.sh`: seeds KV tunables from `config/defaults.env`
- `set_crate_type.sh`: switches crate type between native-test and WASM build modes

Use `make help` for the supported entrypoints.
