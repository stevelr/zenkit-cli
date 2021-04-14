CHANGELOG for zenkit-cli (https://github.com/stevelr/zenkit-cli)

v0.4.5 2021-04-13
list 
- backup can now optionally backup archived/deprecated list items
  with the addition of the `--include-archived` flag.
  When parsing the `*_items.json` file, archived items can be distinguished
  by the non-empty date value for `deprecated_at`.

  Note that this flag backs up archived items from existing lists,
  but _not_ archived lists.

  Tip: If you have a list with the same name as an
  archived list, you may get an error that the list does not
  exist if you lookup the list by name. If you get an error that
  "Resource does not exist" for a list, specify the list by its uuid instead
  of its list name. You can display the uuids with `zk lists`.

v0.4.4

- new config options:
  - Api token can be specified in a config file with '-c' option, or in the
    environment as 'ZENKIT_TOKEN'. The '--token' parameter has been removed to encourage
    best practices of not putting secrets on the command line. 
    Of course, it's still possible to use `ZENKIT_TOKEN="..." zk args ...`
    
  - For subcommands that require a workspace, its name can be specified in the config file
    or from the environment as ZENKIT_WORKSPACE.
    
  - Api endpoint is no longer a cli option, but it can be specified in the config
    file or in the environment as ZENKIT_ENDPOINT.
  
Config file syntax (toml):
```toml
[zenkit]
token = "0000"
workspace = "My Workspace"
```
  - updated dependencies (cfg-if 1.0, bytes 1.0)
  
v0.4.3 2021-01-27

- rebuilt with latest zenkit 0.6.1, which includes fix for
  parsing date fields with no time.

v0.4.2 2021-01-23

- upgrade dependency to zenkit 0.6
- added -w option in webhooks sub-command to set/unset workspace id 
  when creating Webook
- added License files to repo. License 'MIT OR Apache-20' is unchanged.

v0.4.1 2021-01-12

- upgraded dependencies to zenkit 0.5, reqwest 0.11 and tokio 1.0 
