# libc-db-rs #

A very thin CLI client for the [libc.rip API](https://libc.rip/api/) (documented [here](https://github.com/niklasb/libc-database/tree/master/searchengine)).

## Examples ##

Each subcommand corresponds to an endpoint in the public API. For the most part, the client simply prints the JSON response to standard out, which can be further processed with other tools, for example:

* Getting the ID for a libc binary, given its build ID:

```
$ libc-db find --buildid d3cf764b2f97ac3efe366ddd07ad902fb6928fd7 | jq -r '.[].id'
libc6_2.27-3ubuntu1.2_amd64
```

* Getting the address of the `system(3)` function, given a libc ID:

```
$ libc-db dump --symbols "system" libc6_2.27-3ubuntu1.2_amd64 | jq -r '.symbols.system'
0x4f4e0
```

* Getting the download link for a libc, given its ID:

```
$ libc-db find --id libc6_2.27-3ubuntu1.2_amd64 | jq -r '.[].download_url'
https://libc.rip/download/libc6_2.27-3ubuntu1.2_amd64.so
```

* Directly downloading a libc binary, given its build ID, to the current working directory:

```
$ libc-db find --buildid db48bb069f06c7641fdd4e06c6abd4bbafcf1fde --download
https://libc.rip/download/libc6_2.32-0ubuntu2_i386.so -> libc6_2.32-0ubuntu2_i386.so
```

* Getting the address of the `/bin/sh` string for a libc, given the addresses of two other symbols:

```
$ libc_id=$(libc-db find -s 'system: 0x4f4e0, write: 0x110250' | jq -r '.[].id')
$ libc-db dump $libc_id -s 'str_bin_sh' | jq -r '.symbols.str_bin_sh'
0x1b40fa
```

## Command help ##

```
$ libc-db -h
Usage: libc-db <COMMAND>

Commands:
  find  Look up one or more libcs by various attributes. Several attributes can be specified to form an AND filter
  dump  Dump symbols for a given libc ID
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

### Find subcommand (`api/find`) ###

```
$ libc-db find -h
Look up libc by various attributes

Usage: libc-db find [OPTIONS]

Options:
      --md5 <MD5>          Lookup by md5 hash
      --sha1 <SHA1>        Lookup by sha1 hash
      --sha256 <SHA256>    Lookup by sha256 hash
      --buildid <BUILDID>  Lookup by Build ID
      --id <ID>            Lookup by libc ID
  -s, --symbols <SYMBOLS>  Lookup by symbol addresses. Specify with a comma-separated list of colon-separated symbol-address pairs (e.g. 'strncpy: db0, system: 0x4f4e0')
  -d, --download           Do not display results and instead download all libcs that match the query to the current working directory
  -h, --help               Print help information
```

### Dump subcommand (`api/libc/<ID>`) ###

```
$ libc-db dump -h
Dump libc symbols

Usage: libc-db dump [OPTIONS] <ID>

Arguments:
  <ID>  libc ID (e.g. 'libc6_2.27-3ubuntu1.2_amd64')

Options:
  -s, --symbols <SYMBOLS>  Comma-separated list of symbols to dump (e.g. 'strncat, sprintf')
  -h, --help               Print help information
```
