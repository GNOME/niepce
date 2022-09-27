The database upgrade process
============================

On opening the database it will send a
`LibNotification::DatabaseNeedUpgrade(v)` if v isn't the latest
version defined in `DB_SCHEMA_VERSION`.

`Library::perform_upgrade()` will trigger the upgrade process. This
will also copy the database file as a backup.

Writing a new upgrade
---------------------

`db::library::upgrade` is the code for the upgrade process.

`library_to` is a giant loop / match to upgrade one version at a
time. Individual functions perform the upgrade from one version to the
other.

Note: the sqlite schema version isn't related to the
`DB_SCHEMA_VERSION`.
