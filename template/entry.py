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
        "viewport_width": 32,
        "viewport_height": 32,

        # tileset path
        "tileset_path": "tiles.png",
        # Number of tiles along the horizontal axis
        "tileset_width": 40,
        # Number of tiles along the vertical axis
        "tileset_height": 3,

        # Names of each tile in order from left to right, top to bottom.
        "tile_names": [
            "!", "\"", "#", "$", "%", "&", "'", "(", ")", "*",
            "+", ",", "-", ".", "/", "0", "1", "2", "3", "4",
            "5", "6", "7", "8", "9", ":", ";", "<", "=", ">",
            "?", "@", "A", "B", "C", "D", "E", "F", "G", "H",
            "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R",
            "S", "T", "U", "V", "W", "X", "Y", "Z", "[", "\\",
            "]", "^", "_", "`", "a", "b", "c", "d", "e", "f",
            "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
            "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
            "{", "|", "}", "~",
        ]
    }

def __event__(event_type, event_data):
    game_data = pyrite.game_data()

    if event_type == "LOAD":
        game_data["draw_bg_timer"] = 0.0;

        game_data["viewport_width"] = 32
        game_data["viewport_height"] = 32
        game_data["viewport_scale"] = 2
        pass

    if event_type == "BUTTON":
        if event_data["button"] == "ESCAPE":
            pyrite.exit()

        if event_data["transition"] == "PRESSED":
            if event_data["button"] == "Q":
                game_data["viewport_width"] += 1
            if event_data["button"] == "W":
                game_data["viewport_width"] -= 1
            if event_data["button"] == "A":
                game_data["viewport_height"] += 1
            if event_data["button"] == "S":
                game_data["viewport_height"] -= 1
            if event_data["button"] == "Z":
                game_data["viewport_scale"] += 1
            if event_data["button"] == "X":
                game_data["viewport_scale"] -= 1

            pyrite.set_viewport(
                game_data["viewport_width"],
                game_data["viewport_height"],
                game_data["viewport_scale"]
            )

    if event_type == "STEP":
        # colourful background
        game_data["draw_bg_timer"] -= event_data["delta_time"]
        if game_data["draw_bg_timer"] <= 0.0:
            game_data["draw_bg_timer"] += 0.5
            draw_random_bg()

        # instructions
        draw_rect(4, 4, 13, 3, "fill", (0, 0, 0))
        draw_string(5, 5, "Hello, World!")



def draw_string(x, y, s):
    for i, c in enumerate(s):
        pyrite.set_tile((x + i, y), c, (255, 255, 255), (False, False))

def draw_rect(x, y, width, height, tile, color):
    for cx in range(width):
        for cy in range(height):
            pyrite.set_tile((cx + x, cy + y), tile, color, (False, False)) 

def draw_random_bg():
    for x in range(32):
        for y in range(32):
            tile = random.choice(list("!@#$%^&*"))
            color = random.choice([(1,145,135), (146,196,86), (199,228,128), (221,193,85)])
            pyrite.set_tile((x, y), tile, color, (False, False))

