# Up Next

1. Big ole refactor to make shit less embarassing!
2. Make a UI for CHILDREN

## Big picture
- [X] Add remove entity button first
- [ ] Add Name impl to suppress hash field
- [ ] Add System to Show Entity Name on UI header
- [ ] V1 is add/remove entites from child, we don't care about order
  - [ ] Also need 'orphan' button
  - [ ] Need hooks for ChildOf / Children to go update the UI when relationship changes
- [ ] V2 is find a way to care about ordering
  - [ ] Need a system to watch for `Children` mutation so we update the order



# missing scene editor concepts
- [ ] Audio Component player
- [X] Write our own transform handles for 2d
- [ ] A UI for editing `Node` could be pretty neat but might need to wait
        - Is there an MVP here?
- [ ] Need some concept of systems toggles for external systems
- [ ] Need some kind of enable/disable picking filter


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
- [ ] Start refactoring to make use of feathers & theming
  - [?] Move that god awful enum handling code to use somekinda drop down
  - [ ] Put a file / etc set of dropdown menus up top
  - [ ] Investigate documentation tooltips?
  - [ ] Use that delicious new color picker ui to pick some colors
- [ ] Start refactoring to make use of feathers & theming
  - [?] Move that god awful enum handling code to use somekinda drop down
  - [ ] Put a file / etc set of dropdown menus up top
  - [ ] Investigate documentation tooltips?
  - [ ] Use that delicious new color picker ui to pick some colors
- [ ] Start refactoring to make use of feathers & theming
  - [?] Move that god awful enum handling code to use somekinda drop down
  - [ ] Put a file / etc set of dropdown menus up top
  - [ ] Investigate documentation tooltips?
  - [ ] Use that delicious new color picker ui to pick some colors
