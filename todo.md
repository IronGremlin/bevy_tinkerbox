# Up Next

1. Big ole refactor to make shit less embarassing!

## Big picture
 - [X] Clean up editor lib.rs
  - [X] Move UiCtxt and supporting types to its own module
  - [X] Move editor asset stuff to its own module
  - [X] Move scene loading actions to their own widget module
  - [ ] Find something to do with `ImageNodeSansHandle`
- [ ] Start refactoring to make use of feathers & theming
  - [?] Move that god awful enum handling code to use somekinda drop down
  - [ ] Put a file / etc set of dropdown menus up top
  - [ ] Investigate documentation tooltips?
  - [ ] Use that delicious new color picker ui to pick some colors
- [ ] Fixup your silly transform widget
  - [X] Make it track actual transform state on load
  - [ ] Make it populated actual transform state back to ui on edit
    - This actually needs more plumbing, we need to introduce some concept of cascading state update, which needs to be careful not to clobber more specialized state management logic.
  - [ ] Make the buttons in the UI make sense - these should probably be lock buttons not toggle.
  - [ ] Refactor the related code to be not hot garbage also
- [X] Update your text edit stuff to use the better API you introduced with TextureAtlas UI
- [ ] Make a UI for CHILDREN, scaaaarrrry
  - [ ] Add remove entity button first
  - [ ] V1 is add/remove entites from child, we don't care about order
  - [ ] V2 is find a way to care about ordering
  - [ ] Investigate feasibility of not making this an infininitely expanding tree, and just jump to another entry on the flat list - UI is so info dense anyway I'm not sure egui's way of presenting entities makes sense here


# missing scene editor concepts
- [ ] UI for lists of things
- [ ] UI for attaching children?
    - Attaching arbitrary existing entities is out of scope for now. With a file-picker we can effectively attach child scenes, which is powerful enough for most cases.
    - The reason this is different is that we need a VERY different workflow for instantiating and attaching ghost entities - this doesn't fit gracefully into our current event / reflect driven concept
- [X] Sprite UI
        - [X] Texture Atlas UI (make a grid!)
        - [X] Image preview
- [ ] Audio Component player
- [X] Implement a file picker
        - Option A: Fork bevy_file_dialog to support EntityEvents
        - Option B: Implement some user interaction resource to hold a handle to the interacted UI bit while we wait for the global event to trigger, then reference that to put our file path into the UI. 
        - We probably want to fork this lib anyway because we need to update to .18 ASAP, so looks like option A?
    
- [X] Write our own transform handles for 2d
- [ ] A UI for editing `Node` could be pretty neat but might need to wait
        - Is there an MVP here?
- [ ] Need some concept of systems toggles for external systems
- [ ] Need some kind of enable/disable picking filter


# Port over sandbox code into editor package

 - [X] Refactor so that required components aren't brought over automatically, and instead just listed
 - [X] Refactor so that focused entity is supplied instead of being generated
 - [X] Make actual Scene loading / saving code
 - [X] Write some component filtering logic to support scene (de)serialization
 
 # Make standalone sample project editor bin
 
 - [ ] Redefine workflow so that there's like a new scene / load scene / save scene kinda thing happening.
 - [ ] Implement at least some kind of project specific editor code so that we prove out utility early. PlayerAnimation seems like an OK place to start here.

 
 # housekeeping
 
 - [X] crack apart migrated editor code into at least a rough sketch of sane modules
 - [ ] Write project readme


 # back burner
 - [ ] make dev-tools editor screen
 - [ ] We should refactor so that target entities point back to their EntityUiRoot with some kind of relation - should help support scene serialization logic and cleanup traversal
 - [ ] Make another set of hooks for application runtime scene loading, so we can export that under an app-specific plugin (specifically this is for stuff like SpriteShadow)
 - [ ] Refactor cargo features / profiles to make some kinda sense
 - [ ] More module refactoring of the editor lib is definitely overdue, that shit is getting really gnar
 - [ ] Dude write some tests, you should be ashamed
    - dude you moved this to back burner what is wrong with you
 - [ ] Don't actually delay sub camera controls, they belong here. Lmao you moved this to back burner I knew you would
  - [ ] Drag spritemap to translate camera
  - [ ] set UI focus to viewport node
  - [ ] while viewport node is focused, scroll should zoom camera
  - [ ] Make sure both translation and zoom are processed as events so that we can do action mapping later.
- [ ] provide a 'uniform' option for xform scale entry so that when users inevitably totally fuck their aspect ratio by clicking the wrong thing on the widget they can fix their fuckup.
