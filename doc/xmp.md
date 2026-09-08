XMP Documentation
=================

Hubert Figuiere <hub@figuiere.net>

## Merging on import

On import the existing XMP sidecard will override the loaded Exif data
from the file, unless thered is modification date that say it's more
recent. See `npc_fwk::utils::xpm::override_xmp()`

## Niepce namespace

This namespace is to store metadata spefic to Niepce Digital.

Namespace
`niepce::NIEPCE_XMP_NAMESPACE = "http://ns.figuiere.net/ns/niepce/1.0"`
Suggested prefix
`niepce::NIEPCE_XMP_NS_PREFIX = "niepce"`

Values in the "niepce" namespace:

`Flag`   	  : int (-1, 0, 1) # whether the image is flagged or not.
                           # -1 = reject. 1 = pick, 0 = no flag.
`RenderEngine` : string (`tnail`, `ncr`, `rt`) # The rendering engine
                           # `tnail` is not valid other value can be in
                           # the future.

## RawTherapee Compat namespace

This is to store RT .pp3 files into XMP.

Namespace
`metadata::pp3::NS_RT_COMPAT = "http://xmlns.figuiere.net/niepce/rt-compat"`
Suggested prefix
`metadata::pp3::NS_RT_COMPAT_PREFIX = "pp3"`

### RawTherapee settings

Each group in the INI are a property in the XMP in the namespace, and
each property inside the group is a structure field in the
corresponding XMP property.

Group names are normalised to remove spaces and `&`.

### Association

The settings are associated to either the Raw or the JPEG.

`niepce:PP3ForJPEG` and `niepce:PP3ForRaw` are the top level properties that
contain the pp3 properties.

Specific properties are added:

- `niepce:PP3From`: the name of the file it originates from.
- `niepce:PP3Imported`: the date of import.
- `niepce:PP3Modified`: the date of modification. Set on import to
  the same value as `PP3Imported`
- `niepce:PP3Digest`: the sha256 of the file imported.
- `nipece:PP3SyncStatus`: The sync status of the data. `0` mean it is
  synced. This is the value at import. Digest must match. `-1` mean
  XMP is older than the `.pp3`. `1` mean it is more recent.
