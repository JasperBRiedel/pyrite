# Pyrite specification

!! try injecting module into the sys module, and making sure it's set-up correctly.

!!! Need to move to pyo3, rustpython is not in a state that's usable for this project.

## Renderer

* Use opengl es 2
* Full screen quad
* Render using fragment shader 

## API

pyrite.start(config)

pyrite.loop(name, rate)

``` python
configuration = {}

while pyrite.run(configuration):

    if pyrite.load():
        pass

    # Partially event driven
    for event in pyrite.poll_events():
        if event.type == "keyboard":
            pass
        if event.type == "mouse":
            pass
        if event.type == "network":
            pass

    while pyrite.loop("main", 10)
        if pyrite.input("mouse", "left")
            pyrite.exit()

        // viewport is 10 x 10 tiles
        pyrite.camera(10, 10)

        // adds a tile to the scene 
        pyrite.tile(name, x, y)
        pyrite.tile(name, x, y, r, g, b, a)
        pyrite.tile(name, x, y, r, g, b, a, flipx, flipy)

        // flags that the draw buffer is ready to be shown on screen
        pyrite.clear(x, y)
        pyrite.clear(x, y, width, height)


    while pyrite.loop("foo", 1):
        print("1 second passed")
```

# Input

Button constants

Button names;
(keyboard_
- a 
- backspace
- enter
- space
(mouse)
- mouse_left
- mouse_middle
- mouse_right
- mouse_0
Button codes
(keyboard) scan codes
- #1
- #2
- #3
- #4


## Directory structure

\
|-source
| |-entry.py
| \-*.py
|
|-audio
| \-beep.audio*
|
|-tiles
| \-basic_tiles.png
|
|-resources
| \*.*
|
|-release
| |-default_0.1.0_win.exe
| \-default_0.1.0_linux

## Resource packing
All resources are zipped, encrypted and append to the executables footer 

## Third party dependencies
https://crates.io/crates/rustpython
https://crates.io/crates/gl
https://crates.io/crates/image

## Configuration
maybe this could be done via a function call that passes a configuration structure from python
instead? This simplifies the configuration by allowing the developer to use the one language for
everything.

``` python
    
configuration = {
    "application_name": "default",
    "application_version": "0.1.0",
    "engine_mode": "client" | "server",
    "base_grid_size": 20,
    "window_resizeable": "fixed" | "stretch" | "fit" | "fill",
    "window_width": 800,
    "window_height": 600,
    "blend_mode": "halves" | "alternate" | "layer", // how should the engine handle multiple tiles in the same
    space
    "tiles": {
        "basic_tiles.png": {
            "horizontal_tiles": 3,
            "veritical_tiles": 1,
            "tile_names": [
                "grass",
                "dirt",
                "stone",
            ]
        }
    }

}

pyrite.start(configuration)
```

``` toml

[application]
name = "default"
version = "0.1.0"
type = "client" | "server"

[grid]
tile_size = 20 #Each grid space size in pixels

[tilesheets]

tile_pixel_width = 20
tile_pixel_height = 20

[graphics.tilesheets."basic_sprites.png"]
horizontal_tile_count = 3
vertical_tile_count = 1
names = [
"grass", "dirt", "stone"
]


```

