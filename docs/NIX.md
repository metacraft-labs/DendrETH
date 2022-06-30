# Getting started with Nix

Developing and testing all components in the DendrETH projects requires a
diverse set of software packages that must be installed by all contributors:

* Compilers and development libraries for the full set of supported blockchains.
* Blockchain simulation environments.
* Testing frameworks.
* Various project automation tools and utilities such as docker compose, tmux, etc.

If the members of the development team were required to install all these
components manually, setting up the project on a new machine would have
been quite tedious. Updates would be difficult to coordinate and it would
be nearly impossible to ensure that everybody is using the same version
of the required software which creates a lot of potential avenues for the
emergence of the infamous "it works on my machine" problem.

The Nix package manager offers a principled solution for this problem by
making it trivial to describe in code a fully reproducible development
environment where each and every software component used during the build
is pinned to a precise version.

The developers don't have to spend time coordinating upgrades because Nix
can execute them as soon as the file describing the development environment
is changed (naturally, this file is stored in git as part of the project tree).
Besides solving the "it works on my machine" problem, this makes setting up
the project on a new machine trivial.

Furthermore, Nix makes it practical to have multiple projects on a single
machine that use different versions of the required tools. If the DendrETH
project wants to upgrade the Rust complier to a nighly version, this won't
interfere with the build environment of any other projects where all tools
can be pinned to a different version.

You can think about Nix as the "virtualenv for all software". In practice,
working with multiple projects is made extremely easy by the `direnv` tool
which will switch your environment automatically every time you enter a
particular project's directory.

To get started with Nix, you can use the bootstrap script provided in this
repository. It will install Nix and direnv on your machine with the default
recommended settings:

```bash
cd DendrETH
scripts/nix-bootstrap
```

To learn more about Nix, the following articles provides a good introduction:

https://betterprogramming.pub/easily-reproducible-development-environments-with-nix-and-direnv-e8753f456110

The fully-reproducible development environment of the DendrETH project is defined in the following simple file:

https://github.com/metacraft-labs/DendrETH/blob/main/shell.nix

Please note that the precise versions of the referenced packages are pinned through a lock file, also stored in the repo.

