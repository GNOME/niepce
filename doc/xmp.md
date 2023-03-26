XMP Documentation
=================

Hubert Figuiere <hub@figuiere.net>


## Niepce namespace

This namespace is to store metadata spefic to Niepce Digital.

Namespace
`niepce::NIEPCE_XMP_NAMESPACE = "http://ns.figuiere.net/ns/niepce/1.0"`
Suggested prefix
`niepce::NIEPCE_XMP_NS_PREFIX = "niepce";`

Values in the "niepce" namespace:

`Flag`   	  : int (-1, 0, 1) # whether the image is flagged or not.
                           # -1 = reject. 1 = pick, 0 = no flag.
`RenderEngine` : string (`tnail`, `ncr`, `rt`) # The rendering engine
                           # `tnail` is not valid other value can be in
                           # the future.
