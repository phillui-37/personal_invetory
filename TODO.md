# background
have so many resources including
- image (png/jpg/gif)
- ebook (pdf/epub/mobi/azw3)
- video (mp4/mkv)
- game (archive/DLSite/FANZA/Steam/Nintendo)

all these are at different locations including
- computer
    - steamdeck,Mac,windows PC/laptop,Nintendo Switch
- NAS on VPS
- platform
    - bookwalker,kindle
- other web based online reading platform
- portable storage

## difficulties
- do not know the resource owned or not
- hard to find out where the resource store
- forgot resource progress (like reading,playing)
- no tagging for resources grouping and searching
- hard to check metadata for resource

# requirement
- a personal inventory system
- except core feature (db access, os/fs related), all should be conf based dynamic plugin
    - aim to extend or alter behaviour without code change
- every device should have unique id
    - portable storage has its own id due to its nature, so for resource location need to allow free text id input
    - reuse id equals to inherit the id, old device will be delinked
- Nintendo related will all input manually
- backend
    - containerized
    - use as less RAM as possible
    - support PostgreSQL and SQLite
        - need to support doubtless data transfer between two kind of db
    - support fuzzy search of metadata
    - support api key for access control
    - need to change new chapter checking for web based online reading platform
        - should check url and dom only
- frontend
    - provide desktop and mobile
        - cross platform sol is preferred
    - support metadata extraction for resources
        - auto fill extracted metadata
    - DO NOT use electron, one platform one build is acceptable
    - embed webview is required for web based online reading platform progress tracking
        - should check url and dom only
        - auto update reading progress to backends
    - allow batch operation
        - import resources (multi files,dir w/ optional recursive)
        - update metadata
        - copy metadata
    - support steam/DLSite/FANZA check IF POSSIBLE
- share code base is recommended, if cannot share, need to export OpenAPI spec as common interface
    - aims to minimize code size
- no language and framework preference, functional/native/managed/scripts all are welcome with good reasoning