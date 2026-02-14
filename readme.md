# Bevy TinkerBox

A generic scene editor for the bevy game engine.



## FAQ

### What is this?

This is mostly a sandbox for me to explore ideas about interface API design and reflection in Bevy.

I hope it will eventually grow into a useful tool, although it's not likely to ever be a very pretty tool, as I am really exceptionally bad at UI.

For now it is minimally useful for some toy projects.

### What is the dependency foot-print like?

This project attempts to implement as much functionality as possible using vanilla Bevy.

So far there have been two needs that I could not reasonably meet with Bevy:

1. A File selection dialog.
	- This required forking `bevy_file_dialog`
2. Text entry.
	- We rely on `bevy_ui_text_input` for this.
 
Nothing currently on the roadmap would require any other packages.

### What is the current featureset?

Bevy Tinkerbox currently supports:

 - Auto generated component UIs driven by reflection (with some limitations)
 - Basic Support for user supplied UI plugins
 - Minimal 2d scene editing
 - Minimal scene serialization and deserialization
 - Child/Parent heirachical persistence
 
 Bevy Tinkerbox currently has the following severe limitations:
 
  - Relations other than Children/ChildOf will not properly deserialize
  - Scenes which require assets other than Sprite will not load properly
  - The Reflection logic for auto-generating component UIs will fail to generate UI elements for the following Kinds:
	- Maps
	- Sets
	- Arrays
	- Naked Tuples (TupleStructs do work)
  - The auto-generated UI for List types does not support mutating the order of the list elements
  - We don't have a Transform editing widget for 3d yet.
  - We don't have a Transform editing widget for rotations yet.
  - We don't have a Transform editing widget for scale yet.
  - We have no documentation.

When I've checked off those limitations from this list, I will probably consider this to be a useful tool and start actually releasing versioned updates.
Until then you probably shouldn't track this repository directly. If you're mad enough to do so at all.

### Why aren't you just working on the real bevy editor?

Most of what is currently being explored by existing official prototypes is very much in the UX/UI side of things and I am really quite garbage at that.
This project lets me figure out laundry lists of architectural roadblocks so I can contribute meaningfully to discussion.
I -hope- that once it gets far enough along, it can also start exploring the stuff I'm really interested in, which is what the dev story for 3rd party editor widgets is going to look like.

### Are you accepting feedback or contributions?

Sure man, the more the merrier.

Open up an issue or a PR.

### Why haven't you written any tests for this project?

... shhhh.



