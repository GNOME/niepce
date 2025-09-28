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

On the top left you can select the source, Camera or Directory. On the
top right you have a preview of where in the catalog the images will
be imported.

[]

### Camera

There are two different kinds: camera connected over USB and memory
card in readers. For the later case, some camera present themselves as
a card reader and will be handled as such.

Note: The support is provided by libgphoto2 which is a project led by
volunteers, where the support is done without help from camera
vendors.

It is recommended to use a card reader for speed, reliability and
convenience but some cameras and smartphones don't offer that option.

Once the camera is plugged in, click select and it will automatically
query the content to display thumbnails.

### Directory

You can import images from any directory. There are two methods: copy
and in place. Importing from a camera is always copying.

#### Copy import

Use copy import for when you want to copy the files from where they
are to a new location like your central location for all your
images. This is exactly like importing from a camera.

You can sort picture automatically by date.

#### In place

Importing in place mean that you just add the files to the catalog and
theybecome managed by Niepce. This is useful if, for example, you are
importing you archives into Niepce.

In that case the structure of the directories on the storage is kept.

## Destination

The tree view to the right show the destination of the import in the
workspace. Unless it is importing in place, you can select a
destination. It will show the folders created for sorting with the
special folder icon [].

## Sorting

On copy import Niepce can sort images by date into subfolders in the
destination. You can select the sorting and it will be previewed prior
to importing.

[]

There are 5 sorting options. Examples use a picture taken on 28
September 2025:

- No sorting: Pictures are all in the destination
  folder. Example: `Destination/IMG_0001.JPG`.
- `YYYMMDD`: Pictures will be dispatched in a folder with the year
  month and day in the destination
  folder. Example: `Destination/20250928/IMG_0001.JPG`.
- `YYYY/MMDD`: Pictures will be dispatched in a folder with the month
  and day, itself in a folder with the year in the destination
  folder. Example: `Destination/2025/0928/IMG_0001.JPG`.
- `YYYY/MM/DD`: Picture will be sorted by day, in a monthly folder, in
  a yearly folder in the destination folder. Example:
  `Destination/2025/09/28/IMG_0001.JPG`.
- `YYYY/YYYYMMDD`: Pictures will be dispatched in a folder with the
  year month and day, itself in a yearly folder in the destination
  folder. Example: `Destination/2025/20250928/IMG_0001.JPG`.
