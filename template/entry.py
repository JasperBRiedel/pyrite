import pyrite
import random

config = {
    # Application name (used for window tile)
    "application_name": "APPLICATION_NAME",

    # Application version
    "application_version": "0.1.0",

    # Determines the initial window size in pixels
    "window_width": 320,
    "window_height": 320,
    # Determines if the window can be freely resized by the user
    "window_resizable": True,

    # Determines the initial viewport size
    "viewport_width": 10,
    "viewport_height": 10,

    # tileset path
    "tileset_path": "tiles.png",
    # Number of tiles along the horizontal axis
    "tileset_width": 2,
    # Number of tiles along the vertical axis
    "tileset_height": 4,

    # Names of each tile in order from left to right, top to bottom.
    "tile_names": [
        "square",
        "circle_1",
        "circle_2",
        "circle_3",
        "tree",
        "flowers",
        "read_the_docs",
        "pyrite"
    ]
}

def __entry__():
    # All application logic should exist within the engine loop below
    while pyrite.run(config):

        # handle engine events such as input and window resizing
        for event in pyrite.poll_events():
            print(f"Received event: {event}")
            if event["type"] == "button":
                if event["button"] == "escape":
                    pyrite.exit()

        # create a main loop that runs 5 times per second
        while pyrite.timestep("main", 5):
            for x in range(config["viewport_width"]):
                for y in range(config["viewport_height"]):
                    position = (x, y)
                    if should_plant_tree():
                        pyrite.set_tile(
                            position,
                            "tree",
                            pick_plant_color(),
                            (False, False),
                            "flowers",
                            pick_plant_color(),
                            (False, False),
                        );
                    else:
                        pyrite.set_tile(
                            position,
                            "flowers",
                            pick_plant_color(),
                            (False, False),
                        );

            pyrite.set_tile((4, 1), "pyrite", (255, 255, 255), (False, False))
            pyrite.set_tile((5, 1), "read_the_docs", (255, 255, 255), (False, False))

        # create a second loop for other behaviour that runs once per second
        while pyrite.timestep("other", 1):
            print("1 second passed")

def pick_plant_color():
    return random.choice([(1,145,135), (146,196,86), (199,228,128), (221,193,85)])

def should_plant_tree():
    return random.choice([True, False])
