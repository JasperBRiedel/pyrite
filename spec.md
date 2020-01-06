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

    while pyrite.loop("main", 10)
        if pyrite.input("mouse", "left")
            pyrite.exit()

        origin_2d = {"x": 0, "y": 0, "z": 0}

        map = [
            {"tile_name": "grass", "x": 0, "y": 0, "z": 0, "color": [255, 255, 255]}
            {"tile_name": "grass", "x": 0, "y": 1, "z": 0}
            {"tile_name": "dirt", "x": 1, "y": 0, "z": 0}
        ]

        pyrite.draw_tiles(origin_2d, map)

        origin_3d = {"x": 0, "y": 0, "z": 0, "yaw": 0, "pitch": 0}

        pyrtie.draw_voxels(origin_3d, map)

    while pyrite.loop("foo", 1):
        print("1 second passed")
```

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
    "window_resize_mode": "fixed" | "stretch" | "fit" | "fill",
    "window_width": 800,
    "window_height": 600,
    "blend_mode": "halves" | "alternate", // how should the engine handle multiple tiles in the same
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

