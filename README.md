# PLLR

An MVP project build system in rust.

## Usage
`pllr <directory>`

The directory should contain a pllr.json file, in the following format:

```
{
  "items": [
    {
      "get": "git clone https://github.com/left-pad/left-pad.git",
      "source": "left-pad",
      "build": "npm i && npm run test",
      "assets": ["package.json", "perf"],
      "overwrite": true,
      "dest": "lp",
      "children": [<Items>]
    }
  ]
}
```

* `get` specifies the command to fetch resources. Will be run in a fresh temp directory.
* `source` (optional) specifies the directory to change to before executing build commands or copying assets. This is
relative to the temp build directory.
* `build` specifies the command to build or prepare the fetched resources.
* `assets` is an array of files/directories to copy to the output directory.
* `dest` (optional) specifies the directory to copy assets to (relative to and defaults to the root directory).
* `overwrite` (optional) specifies whether to overwrite existing files when copying assets (default is false).
* `children` (optional) is recursive list of items, with their root directory set to the dest of their parent.


## Motivation

Mostly to have a quick play with rust - I'd normally write some shell or python for this kind of task.
