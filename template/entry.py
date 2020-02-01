import pyrite
import random

config = {
    # Application name (used for window tile)
    "application_name": "APPLICATION_NAME",

    # Application version
    "application_version": "0.1.0",

    # Determines the initial viewport size and scale
    "viewport_scale": 2,
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
    width = config["viewport_width"]
    height = config["viewport_height"]
    scale = config["viewport_scale"]

    # All application logic should exist within the engine loop below
    while pyrite.run(config):

        for event in pyrite.poll_events():
            print(f"Received event: {event}")
            if event["type"] == "button":
                if event["button"] == "escape":
                    pyrite.exit()

                if event["transition"] == "pressed":
                    if event["button"] == "q":
                        width += 1
                    if event["button"] == "w":
                        width -= 1
                    if event["button"] == "a":
                        height += 1
                    if event["button"] == "s":
                        height -= 1
                    if event["button"] == "z":
                        scale += 1
                    if event["button"] == "x":
                        scale -= 1
                    pyrite.set_viewport(width, height, scale)

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

            mouse_left = pyrite.button_down("M1")
            mouse_right = pyrite.button_down("M3")
            pyrite.set_tile((4, 1), "pyrite", (255, 255, 255), (mouse_left, mouse_right))
            pyrite.set_tile((5, 1), "read_the_docs", (255, 255, 255), (False, False))

def pick_plant_color():
    return random.choice([(1,145,135), (146,196,86), (199,228,128), (221,193,85)])

def should_plant_tree():
    return random.choice([True, False])
