# jikyuu (時給)

> A tool to estimate the amount of time spent working on a Git repository.

Original code was written in NodeJS called [git-hours](https://github.com/kimmobrunfeldt/git-hours).
This is a port of that code but written in Rust with some quality of life improvements.

**NOTE:** Statistics gathered is only a rough estimate.

## Installation

### Nix

```bash
nix run github:Nate-Wilkins/jikyuu -- --help
```

### Cargo

```bash
cargo install jikyuu
```

## Example

```bash
git clone https://github.com/twbs/bootstrap
cd bootstrap
jikyuu
```

```
+----------------+-------------------------+---------+-----------------+
| Author         | Email                   | Commits | Estimated Hours |
|                |                         |         |                 |
| Mark Otto      | markdotto@gmail.com     | 2902    | 1808.9833       |
| Mark Otto      | otto@github.com         | 2516    | 1709.4          |
| XhmikosR       | xhmikosr@gmail.com      | 1431    | 1612.4667       |
| Chris Rebert   | code@rebertia.com       | 945     | 1019.3          |
| Jacob Thornton | jacobthornton@gmail.com | 826     | 740.35          |
| Mark Otto      | markotto@twitter.com    | 858     | 663.7167        |
| <...>          |                         |         |                 |
|                |                         |         |                 |
| Total          |                         | 16639   | 15041.153       |
+----------------+-------------------------+---------+-----------------+
```

You can associate an author that has used multiple emails in the commit logs with the `--email` (`-e`) option.

```bash
jikyuu -e markotto@twitter.com=markdotto@gmail.com \
       -e otto@github.com=markdotto@gmail.com \
       -e markd.otto@gmail.com=markdotto@gmail.com \
       -e mark.otto@twitter.com=markdotto@gmail.com

```

```
+-----------------+---------------------------+---------+-----------------+
| Author          | Email                     | Commits | Estimated Hours |
|                 |                           |         |                 |
| Mark Otto       | markdotto@gmail.com       | 6880    | 4662.817        |
| XhmikosR        | xhmikosr@gmail.com        | 1431    | 1612.4667       |
| Chris Rebert    | code@rebertia.com         | 945     | 1019.3          |
| Jacob Thornton  | jacobthornton@gmail.com   | 826     | 740.35          |
| Martijn Cuppens | martijn.cuppens@gmail.com | 361     | 508.5           |
| <...>           |                           |         |                 |
+-----------------+---------------------------+---------+-----------------+
```

Use `--format json` (`-f`) to output the data as a JSON array.

```json
[
  {
    "email": "markdotto@gmail.com",
    "author_name": "Mark Otto",
    "hours": 4662.817,
    "commit_count": 6880
  },
  {
    "email": "xhmikosr@gmail.com",
    "author_name": "XhmikosR",
    "hours": 1612.4667,
    "commit_count": 1431
  },

  // ...

  {
    "email": null,
    "author_name": "Total",
    "hours": 14826.803,
    "commit_count": 16639
  }
]
```

## Algorithm

The algorithm for estimating hours is quite simple. For each author in the commit history, do the following:

<br><br>

![](docs/step0.png)

_Go through all commits and compare the difference between
them in time._

<br><br><br>

![](docs/step1.png)

_If the difference is smaller or equal then a given threshold, group the commits
to a same coding session._

<br><br><br>

![](docs/step2.png)

_If the difference is bigger than a given threshold, the coding session is finished._

<br><br><br>

![](docs/step3.png)

_To compensate the first commit whose work is unknown, we add extra hours to the coding session._

<br><br><br>

![](docs/step4.png)

_Continue until we have determined all coding sessions and sum the hours
made by individual authors._

<br>

## Development

Clone and source `.envrc.sh`.

```
git clone git@gitlab.com:nate-wilkins/jikyuu.git
source .envrc.sh && develop
run --help
```

## License

MIT.

## Contributions

| Author  | Estimated Hours |
| ------------- | ------------- |
<%#authors%>| [![<%name%>](https://github.com/<%name%>.png?size=64)](https://github.com/<%name%>) | <p align="right"><%hours%> Hours</p> |
<%/authors%>

## External Resources

- [git-hours](https://github.com/kimmobrunfeldt/git-hours)
- [git2-rs](https://github.com/rust-lang/git2-rs)

