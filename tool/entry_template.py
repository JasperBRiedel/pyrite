import pyrite

config = {
    # Application name
    "application_name": "APPLICATION_NAME",

    # Application version
    "application_version": "0.1.0",

    # Determines the engine mode
    # "client" initialises a window loads graphics logic
    # "server" starts the engine in headless mode ideal for creating a multi-player server
    "engine_mode": "client",

    # The grid size in pixels, this should be the size of the smallest tile
    "base_grid_size": 20,

    # Determines the initial window size in pixels
    "window_width": 800,
    "window_height": 600,

    # Blend mode controls the behaviour when many tiles occupy the same grid space
    # "halves" will display portion of each tile in the space
    # "alternate" will alternate between the tiles every few frames
    "blend_mode": "halves",

    # The default tileset to be loaded on engine start
    "default_tileset": "basic_tiles",

    # Tile set descriptors
    "tiles": {
        # Name of the tile set
        "basic_tiles": {
            # file name of the tile set inside the tilesets directory
            "filename": "basic_tiles.png",
            # Number of tiles along the horizontal axis
            "horizontal_tiles": 3,
            # Number of tiles along the vertical axis
            "vertical_tiles": 1,
            # Names of each tile in order from left to right, top to bottom.
            "tile_names": [
                "grass",
                "dirt",
                "stone"
            ]
        }
    }
}

# All application logic should exist within the engine loop below
while pyrite.run(config):

    if pyrite.load():
        # load stuff here
        pass

    for event in pyrite.poll_events():
        pass

    # create a main loop that runs 60 times per second
    while pyrite.timestep("main", 60):
        # Exit the application if the escape key is pressed
        if pyrite.button_down("escape"):
            print("Escape pressed, exiting...")
            pyrite.exit()

    # create a second loop for other behaviour that runs once per second
    while pyrite.timestep("other", 1):
        print("1 second passed")
