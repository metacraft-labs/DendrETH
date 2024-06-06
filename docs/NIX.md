# Getting started with Nix

Developing and testing all components in the DendrETH projects requires a
diverse set of software packages that must be installed by all contributors:

- Compilers and development libraries for the full set of supported blockchains.
- Blockchain simulation environments.
- Testing frameworks.
- Various project automation tools and utilities such as Docker Compose, tmux, etc.

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
project wants to upgrade the Rust compiler to a nightly version, this won't
interfere with the build environment of any other projects where all tools
can be pinned to a different version.

You can think about Nix as the "virtualenv for all software". In practice,
working with multiple projects is made extremely easy by the [direnv][1]
tool which can switch your environment automatically every time you enter
a particular project's directory.

To install Nix on any Linux distribution or macOS, simply run the following
command:

```bash
sh <(curl -L https://nixos.org/nix/install) --daemon
```

DendrETH is taking advantage of some experimental Nix features such as the
`nix` command the so-called `flakes`, which provide more control when pinning
all dependencies of the project to precise versions. These features must be
[enabled manually][2] after the installation. To learn more about the Nix
flakes, please see the following tutorial:

https://www.tweag.io/blog/2020-05-25-flakes/

The fully-reproducible development environment of the DendrETH project is
defined in the following simple file:

https://github.com/metacraft-labs/DendrETH/blob/main/shell.nix

Please note that the precise versions of the referenced packages are pinned
through a lock file, also stored in the repo.

If you need further and more general introduction to development with Nix,
the following article may be a good start:

https://betterprogramming.pub/easily-reproducible-development-environments-with-nix-and-direnv-e8753f456110

[1]: https://direnv.net/
[2]: https://nixos.wiki/wiki/Flakes#Enable_flakes
