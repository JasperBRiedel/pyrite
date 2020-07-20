import pyrite
import random

def __config__():
    return {
        # Application name (used for window tile)
        "application_name": "APPLICATION_NAME",

        # Application version
        "application_version": "0.1.0",

        # Determines the initial viewport size and scale
        "viewport_scale": 1,
        "viewport_width": 59,
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

    if event_type == "BUTTON":
        if event_data["button"] == "ESCAPE" and event_data["transition"] == "PRESSED":
            pyrite.exit()

    if event_type == "STEP":
        # colourful background
        game_data["draw_bg_timer"] -= event_data["delta_time"]
        if game_data["draw_bg_timer"] <= 0.0:
            game_data["draw_bg_timer"] += 0.5
            draw_random_bg()

        # instructions
        draw_rect(1, 1, 57, 6, "fill", (0, 0, 0))
        draw_string(2, 2, "Welcome to Pyrite!")
        draw_string(2, 3, "1. Open the project directory in your favourite editor")
        draw_string(2, 4, "2. Read the API reference at https://riedel.tech/pyrite")
        draw_string(2, 5, "3. Build an awesome game?!")



def draw_string(x, y, s):
    for i, c in enumerate(s):
        pyrite.set_tile((x + i, y), c, (255, 255, 255), (False, False))

def draw_rect(x, y, width, height, tile, color):
    for cx in range(width):
        for cy in range(height):
            pyrite.set_tile((cx + x, cy + y), tile, color, (False, False)) 

def draw_random_bg():
    for x in range(59):
        for y in range(32):
            tile = random.choice(list("!@#$%^&*"))
            color = random.choice([(1,145,135), (146,196,86), (199,228,128), (221,193,85)])
            pyrite.set_tile((x, y), tile, color, (False, False))

