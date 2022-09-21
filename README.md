# Architectury CLI

A command-line tool that allows [Architectury templates](https://github.com/architectury/architectury-templates) to be downloaded
and extracted into project folders.

This is a very simple tool, and currently only supports the following features:

- Creating project directories from a given template
  - Only supports the latest version for now. Architectury templates are not identified with a version number, so there is no way to identify them other than through the release date or commit hash; neither of these are great options.
- Listing available templates along with their supported Minecraft version(s)
