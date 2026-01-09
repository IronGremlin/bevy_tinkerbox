# Up Next

## Big picture

1. [X] Refactor editor UI with new workflow
    - User creates entity(ies)
    - User selects components
    - UI indicates 'live' but unselected components, with a button to allow over-ride
       - Example: Adding a Text2d should show that a Transform is included, but not as an editable menu item.
       - Reasoning: We should only be serializing those parts of a scene that the user has supplied data for - but knowing what'll be there is still vital.
2. [ ] Make POC for a Sprite UI
3. [ ] Make 2d Transform widget

## lil picture

1. [X] add ui for 'ride-along' components
    - [X] Make presentation for components that exist as a req. for some selected component(s)
    - [X] Swap logic to only generate edit UI for user selected components
    - [X] add a button to allow removing selected components
    - [X] include logic to remove "ride-along" components for removed selected components IFF they aren't present on other selected components. 
    


# missing scene editor concepts
- [ ] UI for lists of things
- [ ] UI for attaching children?
    - Attaching arbitrary existing entities is out of scope for now. With a file-picker we can effectively attach child scenes, which is powerful enough for most cases.
    - The reason this is different is that we need a VERY different workflow for instantiating and attaching ghost entities - this doesn't fit gracefully into our current event / reflect driven concept
- [ ] Sprite UI
        - [ ] Texture Atlas UI (make a grid!)
        - [ ] Image preview
- [ ] Audio Component player
- [ ] Implement a file picker
        - Option A: Fork bevy_file_dialog to support EntityEvents
        - Option B: Implement some user interaction resource to hold a handle to the interacted UI bit while we wait for the global event to trigger, then reference that to put our file path into the UI. 
        - We probably want to fork this lib anyway because we need to update to .18 ASAP, so looks like option A?
    
- [ ] Write our own transform handles for 2d
- [ ] A UI for editing `Node` could be pretty neat but might need to wait
        - Is there an MVP here?
- [ ] Need some concept of systems toggles for external systems
- [ ] Need some kind of enable/disable picking filter


# Port over sandbox code into editor package

 - [ ] Refactor so that required components aren't brought over automatically, and instead just listed
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
 - [ ] Refactor cargo features / profiles to make some kinda sense
 - [ ] More module refactoring of the editor lib is definitely overdue, that shit is getting really gnar
 - [ ] Dude write some tests, you should be ashamed
    - dude you moved this to back burner what is wrong with you
