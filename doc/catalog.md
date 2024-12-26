# Catalog format

The catalog is the main storage for Niepce. It is a sqlite3
[database](database.md) that contain a reference to all the known
files and the metadata.

## File layout

The database file has an extension `.npcat`.

## Thumbnail cache

`ThumbnailCache::path_from_catalog()` will make a path for the
thumbnail cache based on the path to the catalog.
