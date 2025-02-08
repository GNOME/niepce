# Catalog format

The catalog is the main storage for Niepce. It is a sqlite3
[database](database.md) that contain a reference to all the known
files and the metadata.

## File layout

The database file has an extension `.npcat`.

## Thumbnail cache

`ThumbnailCache::path_from_catalog()` will make a path for the
thumbnail cache based on the path to the catalog.

## Reopening

Currently the design to reopen the catalog is too just `exec` the
binary again with the `NIEPCE_OPEN` env set to the catalog path.

This is the simplest method since you can't have more than one open at
the same time.

It is important to call `LibraryClientHost::close()` prior to make
sure the sqlite3 database is closed.
