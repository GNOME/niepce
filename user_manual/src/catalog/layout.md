# Disk layout

This section describe how the catalog is organised on your disk
(storage). This is useful to understand the underlying organisation of
the data that Niepce manage.

The catalog itself is stored in a file with the `.npcat`
extension. This file is a sqlite3 database. In the same location there
is a matching `-thumbcache` directory that contains the preview
cache. This is where images (and video) previews are stored that allow
displayed images quickly. If you need space, you can delete this
directory, it will be recreated as needed.

- `MyCatalog.npcat`
- `MyCatalog-thumbcache`/
