# Design Philosophy

It's important to state the design philosophy for Niepce. This help
guiding the kind of features acceptable for the application and set
the direction.

This simple philosophy is that it should just work by default, and the
result shoud be good. This is implemented via good default choice, and
not an infinity of options. Making these choices can be hard.

## Asset management

One key feature is asset management, i.e. managing your collection of
images in a way that you can organize your workflow and always find
pictures. Some element of inspiration might stem for Adobe Lightroom™
but it doesn't have to be identical.

### Import

Importing catalog from other app is a nice feature. It present some
limitations notably with reusing the edits done, but the organisation
of the library and their metadata.

Importing image from outside is also important, including from your
devices and media.

### Workflow

We chose to be able to build the worflow into the application so you
can do most of your work withing. This include processing images,
exporting them, and a few others.

### Just work

It's important to find the right balance between flexibility and just
working. So managing files while keeping them separate is a goal.

## Processing

What's important with the processing of images is to obtain result
that are expect and to make it easy to edit the parameters.

By default it should try to approach raw rendering to the camera JPEG
processing as much as possible. For example with Fujifilm it should
have rendering profile for the film simulation. Same for other
cameras. If the raw file indicate it's monochrome, the default should
be in monochrome. **This is not easy to achieve given the amount of
cameras**

It should also use the crops. Several cameras offer a crop or a
different aspect ratio and it should be used by default for the
rendering. Again camera settings intent. This is possibly easier to
handle than the colour rendering.

Lens correction should be whenever possible enabled by default. Once
again the variety of camera make this a difficult goal.

Camera support should be as broad as possible.

### Consistency

It's important the processing be consistent across release. This mean
that the pipeline shouldn't behave differently once the image has been
processed.

Adding new algorithms at every release is an anti-goal. Major overhaul
should lead to have an upgrade on demand, and the older pipeline
should still be available. We don't need a zillion demosaicking
algorithms or colour toning.

## User documentation

It's important to have proper user documentation. A goal to have it in
English updated a at the same it as changes seems reasonable. But
effort is to be made to insure localisation can be done in a
reasonable time.

## Updates

The cycle of updates should be in a manner tha we can have
improvements in minor releases and have new thing in major
releases. UI overhauls should not be "just because". Minor releases
should be on a regular basis.

Focusing is fixing things (clearing the debt) is more important than
new fancy feature.

Minor releases
- bug fixes
- speed improvements
- camera specific fix
- new cameras / lens support

Major releases
- new processing options (changes in pipeline)
- new modules
- UI overhaul
- new input methods
