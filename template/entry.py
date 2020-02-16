import pyrite
import random

def __config__():
    return {
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

def __event__(event_type, event_data, game_data):
    if event_type == "load":
        game_data["draw_trees_timer"] = 0.0;

        game_data["viewport_width"] = 10
        game_data["viewport_height"] = 10
        game_data["viewport_scale"] = 2
        pass

    if event_type == "button":
        if event_data["button"] == "escape":
            pyrite.exit()

        if event_data["transition"] == "pressed":
            if event_data["button"] == "q":
                game_data["viewport_width"] += 1
            if event_data["button"] == "w":
                game_data["viewport_width"] -= 1
            if event_data["button"] == "a":
                game_data["viewport_height"] += 1
            if event_data["button"] == "s":
                game_data["viewport_height"] -= 1
            if event_data["button"] == "z":
                game_data["viewport_scale"] += 1
            if event_data["button"] == "x":
                game_data["viewport_scale"] -= 1

            pyrite.set_viewport(
                game_data["viewport_width"],
                game_data["viewport_height"],
                game_data["viewport_scale"]
            )

    if event_type == "step":
        game_data["draw_trees_timer"] -= event_data["delta_time"]

        if game_data["draw_trees_timer"] <= 0.0:
            game_data["draw_trees_timer"] += 0.5

            for x in range(10):
                for y in range(10):
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

