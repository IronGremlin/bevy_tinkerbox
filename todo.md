# Up Next

## Big picture

1. [X] Refactor editor UI with new workflow
    - User creates entity(ies)
    - User selects components
    - UI indicates 'live' but unselected components, with a button to allow over-ride
       - Example: Adding a Text2d should show that a Transform is included, but not as an editable menu item.
       - Reasoning: We should only be serializing those parts of a scene that the user has supplied data for - but knowing what'll be there is still vital.
2. [X] Make POC for a Sprite UI
3. [X] Make 2d Transform widget

## lil picture
 - [X] Add marker components to _relate_ (because just mark isn't enough) texture atlas preview visualizations so they can be despawned when they have to be updated.
 - [X] Add Grid UI 
      - Drag widget for box size
      - Text entry for padding
      - `+ / -` buttons for both rows and colums
  - [X] Crack out this functionality into a widget that can be invoked by a texture_altas button
     - Don't forget - UI for texture atlas still needs an index into the layout.
     - Also don't forget - camera etc. needs to be gracefully cleaned up.
  - [X] Actually wire this up so that it saves this to a sprite.
  - [ ] For later: SubCamera / scroll controls for large images.

## lil er picture
- [X] Refactor dragsnapstate stuff so it can be generalized as a global observer
- [X] implement offset drag
- [~] do something about bounds checking on the sprite atlas? Maybe?
- [X] Wireup text-based ui controls
- [ ] Don't actually delay sub camera controls, they belong here.
  - [ ] Drag spritemap to translate camera
  - [ ] set UI focus to viewport node
  - [ ] while viewport node is focused, scroll should zoom camera
  - [ ] Make sure both translation and zoom are processed as events so that we can do action mapping later.

### Sprite UI
1. [X] Wire up file picker dialog
2. [X] Make mvp UI for sprite
   - Just build a minimum to wire up the handle with some defaults to get it to show.
   - This requires loading image as asset, then assessing texture_descriptor.size to fill sprite .rect property
3. [X] Figure out how to do architecture for TextureAtlas preview -
    - This gets weird because we can't just operate in world-space
    - might need to mess around with doing some kinda render-layer stuff and put the UI in a separate canvas - can default picking work through an image layer canvas? I'd imagine no...
    - kicking the can on this to preserve momentum since MVP ended up easier than I'd anticpated.
    
### Xform wiget
1. spawn basic shape widget
2. add (basic) editor scene-view camera logic/controls
  - Remember to care about UI focus!
  - Maybe take advantage of this to do some groundwork for more advanced display/render target stuff later so UI is truly separate and we leave space for advanced picking logics later
3. Figure logic for locking 2d widget scale to camera scale so that relative widget size is fixed to camera scale
4. Our very first setting, "transform widget size", so that the user can tweak the apparent size of the xform widget handles
5. do the actual drag controls for the widget
6. provide a 'uniform' option for xform scale entry so that when users inevitably totally fuck their aspect ratio by clicking the wrong thing on the widget they can fix their fuckup.

    


# missing scene editor concepts
- [ ] UI for lists of things
- [ ] UI for attaching children?
    - Attaching arbitrary existing entities is out of scope for now. With a file-picker we can effectively attach child scenes, which is powerful enough for most cases.
    - The reason this is different is that we need a VERY different workflow for instantiating and attaching ghost entities - this doesn't fit gracefully into our current event / reflect driven concept
- [X] Sprite UI
        - [X] Texture Atlas UI (make a grid!)
        - [X] Image preview
- [ ] Audio Component player
- [ ] Implement a file picker
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
 - [ ] Refactor so that focused entity is supplied instead of being generated
 - [ ] Make actual Scene loading / saving code
 - [ ] Write some component filtering logic to support scene (de)serialization
 
 # Make standalone sample project editor bin
 
 - [ ] Redefine workflow so that there's like a new scene / load scene / save scene kinda thing happening.
 - [ ] Implement at least some kind of project specific editor code so that we prove out utility early. PlayerAnimation seems like an OK place to start here.

 
 # housekeeping
 
 - [X] crack apart migrated editor code into at least a rough sketch of sane modules
 - [ ] Write project readme


 # back burner
 - [ ] make dev-tools editor screen
 - [ ] We should refactor so that target entities point back to their EntityUiRoot with some kind of relation - should help support scene serialization logic and cleanup traversal
 - [ ] Refactor cargo features / profiles to make some kinda sense
 - [ ] More module refactoring of the editor lib is definitely overdue, that shit is getting really gnar
 - [ ] Dude write some tests, you should be ashamed
    - dude you moved this to back burner what is wrong with you
