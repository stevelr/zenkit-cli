Use Zenkit from the command-line and scripts.
Read and update workspaces, lists, items, fields, and webhooks;
and perform backups to json.

## Installation

Install with `cargo install zenkit-cli`. The program name is `zk`
(usually installed to $HOME/.cargo/bin).
Instructions for installing cargo are
[here](https://doc.rust-lang.org/cargo/getting-started/installation.html)

Set the environment variable `ZENKIT_API_TOKEN` to your api token, which
you can obtain (even for the free-tier plan) from your Zenkit account.

Optional: To avoid re-typing `-w WORKSPACE` for every command for 
the most-used workspace,
set the environment variable `ZENKIT_WORKSPACE` to the workspace name. 
The `-w WORKSPACE` option always overrides `ZENKIT_WORKSPACE`.

## zk Usage

Use `zk -h` for help.

For the commands below, the parameter values for
`workspace`, `list`, or `field` may be an object's id (int),
uuid, or display name. Values containing spaces or symbols should be
quoted.

All commands except `workspaces` require a `-w workspace` parameter or
require the environment variable `ZENKIT_WORKSPACE` to contain a
workspace name. The -w option is omitted below for brevity.

  - Show help</br>`zk -h/--help`

  - Workspace commands

    - Show all workspaces and lists (accessible by your user)</br>
    `zk workspaces`</br>
      Output columns (tab-separated):
      - W/L:  workspace or list
      - id:   object id (positive int)
      - uuid: object uuid
      - name: object name

    - Show users in workspace </br>`zk users`</br>
      Output columns (tab-separated):
      - id
      - uuid
      - name

    - Show lists in workspace </br>`zk lists`</br>
      Output columns (tab-separated):
      - id
      - uuid
      - name

  - List commands

    - Show items in a list</br> `zk items -l list`</br>
      Output columns (tab-separated):
      - id
      - uuid
      - name

  - List field/schema commands

    - Show fields for a list </br>`zk fields -l list`</br>
      Output columns (tab-separated):
      - id
      - uuid
      - name

    - Show choice values for a field</br>`zk choices -l list -f field`</br>
      Output columns (tab-separated):
      - id
      - name

    - Describe field</br>`zk field -l list -f field`</br>
      Output format: Text object dump

  - Item commands

    - Show item detail</br>`zk item -l list -i item_num`</br>
      Output format: object dump (text)

    - Set field value</br>
      `zk set -l list -i item_num -f field [-t text] [-v value | -F file]`</br>

      The value can be specified on the command-line (-v) or from a file
	  (-F).
      
	  For a field of type person, the value may be either the person's
	  uuid or their display name (case-insensitive).
	  For a field of type choice (category), the value
	  may be the choice id, uuid, or display name(case-sensitive). For a field of type
	  reference, the value must be the uuid of the related object.

    - Create item</br>
    `zk create -l list -F field=value -F field=value ...jj`</br>

	  Values may be of the format described above for "Set field value".
	  Field names may be id, uuid, or display name (case-sensitive).

    - Add comment to an item</br>`zk comment -l list -i item -c comment`

  - Webhooks

    - Add a webhook</br>
      `zk webhook --type triggger-type --url url [ OPTIONS ]`

    - Delete webhook</br>
      `zk delete-webhook --webhook webhook`

    - List webhooks</br>
      `zk list-webhooks`
  
  - Backup
    - Backup lists and field definitions to json files</br>
      `zk backup -o output_dir [ -l list ]`</br>
      If no list is specified, all lists in the workspace are backed up.

