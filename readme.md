**Pyrite** has been discontinued. As promised to those who purchased a commercial build of the software, I have released the source code. You are now free to continue using and modifying Pyrite according to your needs.

# Pyrite Game Engine

Pyrite is a simple game engine designed from the ground up to enable text and tile based game development, a style popular in the indie rougelike scene.

[Visit the itch.io page for builds and more information](https://riedel-tech.itch.io/pyrite)

## Key Features

Pyrite boasts a wide range of features to streamline your game development process:

-   **Simple API**: With only a handful of functions to learn, you can quickly get started with Pyrite.
-   **Hardware Accelerated Graphics**: Enjoy smooth graphics with hardware acceleration.
-   **Effortless Toolchain**: Pyrite provides a straightforward toolchain for creating, building, and running projects.
-   **One-Step Game Builds**: Create a game executable with a single command.
-   **Dynamic Window Adjustments**: Modify window settings and scale your game on the fly.
-   **Full Mouse and Keyboard Support**: Take full advantage of comprehensive input control.
-   **Color Modifiers**: Apply color modifiers to grayscale tiles for stunning visual effects.
-   **Two Render Layers**: Use two render layers for creating more complex scenes.
-   **Versatile Architectures**: Pyrite supports both polling and event-driven architectures.
-   **Automated Tile Indexing**: Enjoy automated tile indexing and name assignment for convenience.
-   **Libtcod Workflow**: Follow a workflow similar to that of libtcod, making development more intuitive.
-   **Offline Documentation**: Comprehensive offline documentation is included with the download for reference.

## Getting Started

Here's a step-by-step guide to getting started with Pyrite:

1. **Download and Extract**: Begin by downloading the ZIP archive and extracting its contents.

2. **Run the "pyrite-tool" Executable**: Choose the appropriate executable for your platform.

3. **Create a New Project**: Use the following command to create a new project: `new project-name`.

4. **Open the Project Directory**: Your project directory's full path will be displayed in the tool window. Open it in your favorite text editor.

5. **Develop Your Game**: Get creative and start building your game (be sure to consult the provided documentation file).

6. **Run Your Project**: Test your game by running the following command: `run project-name`.

7. **Build for Distribution**: When you're ready to create a game executable for distribution, use the following command: `build project-name`. The build will be placed under the "builds" directory.

**Linux Note**: On Linux systems, ensure that Python 3 is installed for Pyrite to work correctly.

---

# Pyrite Game Engine API Documentation

Welcome to the Pyrite Game Engine API documentation. This section provides comprehensive information about the engine's functions and constants to assist you in your game development journey.

## Table of Contents

1. [Pyrite Configuration](#pyrite-configuration)
    - [config() - Pyrite configuration callback](#config-pyrite-configuration-callback)
2. [Engine Life Cycle](#engine-life-cycle)
    - [event() - Engine life cycle callback](#event-engine-life-cycle-callback)
    - [game_data() - The preferred way of storing global game state](#game-data-the-preferred-way-of-storing-global-game-state)
    - [delta_time() - Consistent timing logic](#delta-time-consistent-timing-logic)
    - [exit() - Initiate a graceful exit of the engine](#exit-initiate-a-graceful-exit-of-the-engine)
3. [Input Handling](#input-handling)
    - [mouse_position() - Get the location of the mouse](#mouse-position-get-the-location-of-the-mouse)
    - [button_down() - Get the down state of a button input](#button-down-get-the-down-state-of-a-button-input)
4. [Viewport and Tile Management](#viewport-and-tile-management)
    - [set_viewport() - Configure the dimensions and scale of the window](#set-viewport-configure-the-dimensions-and-scale-of-the-window)
    - [set_tile() - Set tile draw properties](#set-tile-set-tile-draw-properties)
    - [clear() - Clear the scene](#clear-clear-the-scene)
5. [Resource Management](#resource-management)
    - [resource_read() - Loading packaged resources](#resource-read-loading-packaged-resources)
    - [resource_exists() - Check if packaged resources exist](#resource-exists-check-if-packaged-resources-exist)
6. [Audio Playback](#audio-playback)
    - [play_audio() - Play an audio file](#play-audio-play-an-audio-file)
    - [pause_audio() - Pause an audio file](#pause-audio-pause-an-audio-file)
    - [stop_audio() - Stop playing an audio file](#stop-audio-stop-playing-an-audio-file)
    - [volume_audio() - Set the volume of a playing audio file](#volume-audio-set-the-volume-of-a-playing-audio-file)
7. [Event Types and Data](#event-types-and-data)
    - [Constants - Events and Buttons](#constants-events-and-buttons)
8. [Button Constants](#button-constants)
9. [Scancode Constants](#scancode-constants)

---

## Pyrite Configuration

### config() - Pyrite Configuration Callback

Initialize the engine with the returned configuration structure. This function must be defined in the entry module.

```python
def __config__:
    return {
        "application_name": application_name,
        "application_version": application_version,
        "viewport_scale": viewport_scale,
        "viewport_width": viewport_width,
        "viewport_height": viewport_height,
        "tileset_path": tileset_path,
        "tileset_width": tileset_width,
        "tileset_height": tileset_height,
        "tile_names": tile_names
    }
```

-   `application_name`: Used to identify the application and set the window title.
-   `application_version`: Should be a version number complying with semantic versioning.
-   `viewport_scale`: Scale factor provided as a positive integer.
-   `viewport_width`: Initial width of the viewport in tiles.
-   `viewport_height`: Initial height of the viewport in tiles.
-   `tileset_path`: Name of the tileset file, including the extension.
-   `tileset_width`: Horizontal tile count in the tileset.
-   `tileset_height`: Vertical tile count in the tileset.
-   `tile_names`: An array of tile names to be assigned to tiles in left-to-right, top-to-bottom order. Fully transparent tiles won't be indexed.

## Engine Life Cycle

### event() - Engine Life Cycle Callback

The event callback is invoked by the engine to progress the game state. Use this function to update the game state based on inputs and elapsed time. It must be defined in the entry module.

```python
def __event__(event_type, event_data):
    pass
```

-   `event_type`: A string containing an event type constant. See the constants section for details.
-   `event_data`: A dictionary containing event-specific data. See the events section for details.

### game_data() - The Preferred Way of Storing Global Game State

Access the global game state/data dictionary.

```python
pyrite.game_data
```

This returns a reference to a global dictionary intended for holding game state/data. Using this mechanism is preferred over global variables.

### delta_time() - Consistent Timing Logic

Due to the nature of the engine update loop and how some game logic may exceed the allocated execution time of step events, it's important to take delta time (the time since the last update) into consideration when dealing with time-sensitive calculations.

```python
pyrite.delta_time()
```

This function returns the time in seconds since the last step event. It will return 0.0 if called outside of the step event. The returned value can be accumulated to form a timer of seconds elapsed.

### exit() - Initiate a Graceful Exit of the Engine

Instruct the engine to gracefully exit.

```python
pyrite.exit()
```

## Input Handling

### mouse_position() - Get the Location of the Mouse

Get the location of the mouse in tile coordinates.

```python
pyrite.mouse_position()
```

This function returns the position on the X and Y axis in tile coordinates.

-   `x`: Position on the X axis in tile coordinates.
-   `y`: Position on the Y axis in tile coordinates.

## Viewport and Tile Management

### set_viewport() - Configure the Dimensions and Scale of the Window

Set the width and height of the window in tile increments and the tile scale factor.

```python
pyrite.set_viewport(width, height, scale)
```

-   `width`: Width in tiles of the window. Must be a whole number.
-   `height`: Height in tiles of the window. Must be a whole number.
-   `scale`: Scale factor of the tiles. Must be a whole number.

### set_tile() - Set Tile Draw Properties

Set the display properties of the top layer tile in the scene.

```python
pyrite.set_tile(x, y, name, red, green, blue, flip_x, flip_y)
```

-   `(x, y)`: The x and y coordinate tuple of the tile to be set.
-   `name`: The name of the tile sprite as defined in the configuration structure returned by `__config__()`.
-   `(red, green, blue)`: The RGB color tuple, multiplies the tile colors by the modifier values, allowing color shifting and coloring of grayscale sprites.
-   `(flip_x, flip_y)`: Tile sprite flip tuple, boolean value determines if the tile should be flipped on that axis.

### clear() - Clear the Scene

It's generally better for performance to just update the tiles that have changed, but in some cases, it might become necessary to just clear the scene before rendering the next frame.

```python
pyrite.clear()
```

## Resource Management

### resource_read() - Loading Packaged Resources

Read a packaged resource into a string and return it.

```python
pyrite.resource_read(name)
```

-   `name`: Name of a packaged file including the file extensions.

This function returns the file data as a string.

### resource_exists() - Check if Packaged Resources Exist

Check a packaged resource to see if it exists.

```python
pyrite.resource_exists(name)
```

-   `name`: Name of a packaged file including the file extensions.

This function returns `True` if the file exists.

## Audio Playback

### play_audio() - Play an Audio File

Start playing an audio file.

```python
pyrite.play_audio(name)
```

-   `name`: Name of a packaged audio file including the file extension.

### pause_audio() - Pause an Audio File

Pause the specified audio file.

```python
pyrite.pause_audio(name)
```

-   `name`: Name of a packaged audio file including the file extension.

### stop_audio() - Stop Playing an Audio File

Stop the audio track.

```python
pyrite.stop_audio(name)
```

-   `name`: Name of a packaged audio file including the file extension.

### volume_audio() - Set the Volume of a Playing Audio File

Set the volume of a currently playing track.

```python
pyrite.volume_audio(value)
```

-   `value`: Sample volume modifier.

## Event Types and Data

### Events

Event constants are used to select the various event types that can be raised by the engine:

-   `LOAD`: Raised when the engine is ready for the game to load.
-   `BUTTON`: Raised when the operating system reports a keyboard or mouse button transition.
-   `SCROLL`: Raised when the operating system reports a scroll wheel change from the mouse.
-   `TEXT`: Raised when text input is received from the keyboard.
-   `STEP`: Repeatedly raised at approximately 60Hz, used for real-time logic and game updates.
-   `EXIT`: Raised when the engine is instructed to exit (e.g., window closed or exit function called).

## Buttons

Button name constants are passed as strings to various functions. These constants are always uppercase and represent various buttons, both keyboard and mouse. Below is a comprehensive list of all the named button constants:

-   `MOUSE_LEFT`
-   `MOUSE_MIDDLE`
-   `MOUSE_RIGHT`
-   `MOUSE_1`
-   `MOUSE_2`
-   `MOUSE_3`
-   `MOUSE_4`
-   `MOUSE_5`
-   `MOUSE_6`
-   `NUMBER0`
-   `NUMBER1`
-   `NUMBER2`
-   `NUMBER3`
-   `NUMBER4`
-   `NUMBER5`
-   `NUMBER6`
-   `NUMBER7`
-   `NUMBER8`
-   `NUMBER9`
-   `A`
-   `B`
-   `C`
-   `D`
-   `E`
-   `F`
-   `G`
-   `H`
-   `I`
-   `J`
-   `K`
-   `L`
-   `M`
-   `N`
-   `O`
-   `P`
-   `Q`
-   `R`
-   `S`
-   `T`
-   `U`
-   `V`
-   `W`
-   `X`
-   `Y`
-   `Z`
-   `ESCAPE`
-   `F1`
-   `F2`
-   `F3`
-   `F4`
-   `F5`
-   `F6`
-   `F7`
-   `F8`
-   `F9`
-   `F10`
-   `F11`
-   `F12`
-   `F13`
-   `F14`
-   `F15`
-   `F16`
-   `F17`
-   `F18`
-   `F19`
-   `F20`
-   `F21`
-   `F22`
-   `F23`
-   `F24`
-   `SNAPSHOT`
-   `SCROLL`
-   `PAUSE`
-   `INSERT`
-   `HOME`
-   `DELETE`
-   `END`
-   `PAGE_DOWN`
-   `PAGE_UP`
-   `LEFT`
-   `UP`
-   `RIGHT`
-   `DOWN`
-   `BACK`
-   `RETURN`
-   `SPACE`
-   `COMPOSE`
-   `CARET`
-   `NUMLOCK`
-   `NUMPAD0`
-   `NUMPAD1`
-   `NUMPAD2`
-   `NUMPAD3`
-   `NUMPAD4`
-   `NUMPAD5`
-   `NUMPAD6`
-   `NUMPAD7`
-   `NUMPAD8`
-   `NUMPAD9`
-   `BACKSLASH`
-   `CALCULATOR`
-   `CAPITAL`
-   `COLON`
-   `COMMA`
-   `CONVERT`
-   `DECIMAL`
-   `DIVIDE`
-   `EQUALS`
-   `GRAVE`
-   `KANA`
-   `KANJI`
-   `LEFT_ALT`
-   `LEFT_BRACKET`
-   `LEFT_CONTROL`
-   `LEFT_SHIFT`
-   `LEFT_SUPER`
-   `MAIL`
-   `MEDIA_SELECT`
-   `MEDIA_STOP`
-   `MINUS`
-   `MULTIPLY`
-   `MUTE`
-   `MY_COMPUTER`
-   `NAV_FORWARD`
-   `NAV_BACKWARD`
-   `NEXT_TRACK`
-   `NO_CONVERT`
-   `NUMPAD_COMMA`
-   `NUMPAD_ENTER`
-   `NUMPAD_EQUALS`
-   `PERIOD`
-   `PLAY_PAUSE`
-   `POWER`
-   `PREV_TRACK`
-   `RIGHT_ALT`
-   `RIGHT_BRACKET`
-   `RIGHT_CONTROL`
-   `RIGHT_SHIFT`
-   `RIGHT_SUPER`
-   `SEMICOLON`
-   `SLASH`
-   `SLEEP`
-   `STOP`
-   `SUBTRACT`
-   `TAB`
-   `UNDERLINE`
-   `VOLUME_UP`
-   `VOLUME_DOWN`

## Scancode Constants

Scancodes should be used when you care more about the location of the button than the symbol/meaning of the button. Here are some examples:

-   `K_0`
-   `K_1`
-   `K_2`
-   ...
-   `K_997`
-   `K_998`
-   `K_999`

## License

This source code is provided as a courtesy to those who had purchased paid commercal copy of the engine so that they can continue developmet and make adjustments as necessary. This follows up on the initial promise that the source would be released when the project reaches end of life.

```
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND NON-INFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES, OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT, OR OTHERWISE, ARISING FROM, OUT OF, OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
```
