# Database description

Author: Hubert Figuiere <hub@figuiere.net>

Unless mentionned, this is starting in version 6.

## Administration table.

Contain various parameters, including options.

Table name: `admin`

| Column  | Description  |
|---------|--------------|
| `key`   | Key (unique) |
| `value` | Value        |

The known keys are:

| Key                              | Description                                |
|----------------------------------|--------------------------------------------|
| `version`                        | The version of the database. Current = 13. |
| `prefs.last_dir_import_location` | The last directory imported                |
| `prefs.dir_import_copy`          | Copy when importing directory              |
| `prefs.dir_import_recursive`     | Recursive directory import                 |
| `prefs.last_importer`            | Last importer selected (dir or camera)     |
| `prefs.base_import_dest_dir`     | Base directory for import destination      |
| `prefs.catalog-window`           | State of the Catalog window (NiepceWindow) |
| `prefs.import_sorting`           | The import sorting.                        |

The key values aren't part of the version of the database schema,
except `version`.

[ version = 13 ]
`key` is primary key in table.

Preference keys are mostly free form and should have the prefix
`prefs.`.

## Files

Files in the catalog

Table name: `files`

| Column        | Description                                                   |
|---------------|---------------------------------------------------------------|
| `id`          | Unique ID in the database                                     |
| `main_file`   | ID in fsfiles for the main file.                              |
| `name`        | The (display) name of the file                                |
| `parent_id`   | The ID on the containing folder (= folders.id)                |
| `orientation` | The Exif orientation of the file                              |
| `file_type`   | The file type. See [`libfile::FileType`] for possible values. |
| `file_date`   | The file date, likely shooting date from Exif (time_t)        |
| `rating`      | The file rating (0-5)                                         |
| `label`       | The label (labels.id)                                         |
| `flag`        | The file flag. (-1 reject, 0 none, +1 flagged)                |
| `import_date` | The date of import in the database (time_t)                   |
| `mod_date`    | The date modified (time_t)                                    |
| `xmp`         | The XMP blob                                                  |
| `xmp_date`    | The date the XMP is rewritten on disk (time_t)                |
| `xmp_file`    | The id of the fsfile that represent the XMP (int)             |
| `jpeg_file`   | The id of the JPEG for RAW+JPEG. (int)                        |

## Filesystem files

Filesystem files in the catalog

Table name: `fsfiles`

| Column | Description               |
|--------|---------------------------|
| `id`   | Unique ID in the database |
| `path` | The absolute path         |

## Sidecars

Sidecars are backed to a `fsfiles` and attached to a `file` (excepted
XMP and JPEG alternate).  [ version = 7 ]

Table name: `sidecars`

| Column      | Description                                                   |
|-------------|---------------------------------------------------------------|
| `file_id`   | ID of the file.                                               |
| `fsfile_id` | ID of the fsfile sidecar.                                     |
| `ext`       | String: the extension (no dot). [ version = 8 ]               |
| `type`      | Sidecar type. 1 = Live, 2 = Thumbnail. (See the Sidecar enum) |

## Folders

Folders for the catalog "storage". A folder map to a file system directory.

Table name: `folders`

| Column      | Description                                                |
|-------------|------------------------------------------------------------|
| `id`        | Unique ID in the database                                  |
| `path`      | The path of the root folder, NULL otherwise.               |
| `name`      | The path component.                                        |
| `vault_id`  | The vault ID (unused) 0 = no vault (= `vaults.id`)         |
| `locked`    | Can't be deleted if non 0. For special folders.            |
| `virtual`   | Type of virtual item. See [`libfolder::FolderVirtualType`] |
| `parent_id` | The ID of the parent (= `folders.id`). 0 = root folder     |
| `expanded`  | 1 if expanded, 0 if not. Default = 0. Unused for now.      |

[ version = 12 ]
Folders are unique on (`name`, `parent_id`).

`path`: A relative path is relative from the catalog file. Otherwise
it's absolute. It is updated automatically based on the `parent_id`
and root folders.

## Keywords

Keywords are defined in a table, and linked to files in the other

Table name: `keywords`

| Column      | Description                                         |
|-------------|-----------------------------------------------------|
| `id`        | Unique ID in the database                           |
| `keyword`   | The text of the keyword                             |
| `parent_id` | The parent keyword. 0 = top level (= `keywords.id`) |

The `file` / `keyword` association is done on a `keywording` table.

Table name: `keywording`

| Column       | Description                                 |
|--------------|---------------------------------------------|
| `file_id`    | The file ID it is linked to (= `files.id`)  |
| `keyword_id` | The keyword ID associated (= `keywords.id`) |

There shouldn't be more than one pair of identical (`file_id`, `keyword_id`)

## Labels

Labels for the file. There are very few of these.

Table name: `labels`

| Column  | Description                            |
|---------|----------------------------------------|
| `id`    | The ID of the label                    |
| `name`  | The name of the label (user displayed) |
| `color` | The RGB8 color in "R G B" format.      |

## Albums

Albums contain files (added in version 10)

Table name: `albums`

| Column      | Description                            |
|-------------|----------------------------------------|
| `id`        | The ID of the album                    |
| `name`      | The name of the album (user displayed) |
| `parent_id` | The parent album. -1 means on the top  |

Table name: `albuming`

| Column     | Description               |
|------------|---------------------------|
| `file_id`  | The file in the album.    |
| `album_id` | The album the file is in. |

## Update queue

The update queue for XMP. When an XMP is changed in the DB it is
queued in the table.


Table name: `xmp_update_queue`

| Column | Description        |
|--------|--------------------|
| `id`   | File ID to update. |

## Vaults

Vaults are storage location for files. Currently unimplemented

Table name: `vaults`

| Column | Description                |
|--------|----------------------------|
| `id`   | Unique ID in the database  |
| `path` | Absolute path of the vault |
