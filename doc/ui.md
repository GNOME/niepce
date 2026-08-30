# UI concepts

## Selection

Selecting pictures is an essential part of the user interaction.

It is handled by the SelectionController which itself wraps around the
gtk::MultiSelection from the toolkit.

### Order

Selected item are added to a set in order.

[TODO] What happen to moving in the order of the list?

### Behaviour

What happen with moving the selection left or right ?

If the selection is only one item, it just moves it.

[TODO] If the selection is multiple items, then moving the selection
should move the primary within the selection range.

### Primary

[TODO] Some functions don't work with more than one image. Like the
darkroom module where you edit one image at a time. So there is need
to have one item that is the primary.

### Metadata

When displaying metedata it's important to acknowledge the mixed
state, one where the multiple selection as different values for a
specific metadata. This is the type `PropertyValue::Mixed`. When
changing it will unify the value across the selection.
