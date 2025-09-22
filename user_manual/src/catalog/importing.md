# Importing

Importing images (and videos) into your catalog is the first step to
be able to edit them.

Niepce support importing from cameras connected via USB, from memory
card, or directly from a drive, including if you alread have the
images on your computer.

The catalog will retain where the pictures are and help you organise
them. So when importing images from a drive, you can chose to
reference the images directly, or copy them in a new location. When
importing from a camera or a memory card, the files get copied into a
location you choose.

## File bundles

Images are imported as bundles, where you have the image file(s) and
the associated sidecars. If you shoot raw + JPEG (or HEIF) then the
two files represent one image, and then the XMP sidecar if it exists
is added. In the case of iPhone Live pictures, the corresponding movie
file is attached to it. All of that is a bundle. The term "bundle"
isn't used anywhere. It's just images with sidecars.

Video files are included too.

## Sources

There are two sources: cameras or directory. Cameras include cameras
connected with USB or memory cards in a card reader. Import directory
is usually to import image you already have on your computer (or a
drive connected to it).

To start import click on import on the top left of the catalog window.

[]

Then you have the import dialog:

[]

On the top left you can select the source, Camera or Directory.

### Camera

There are two different kinds: camera connected over USB and memory
card in readers. For the later case, some camera present themselves as
a card reader. The support is provided by libgphoto2 which is a
project led by volunteers, where the support is done without help from
camera vendors.

It is recommended to use a card reader for speed reliability and
convenience but some camera and smartphones don't offer that option.

Once the camera is plugged, click select and it will automatically
query the content to display thumbnails.

### Directory

