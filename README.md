# Pimp-My-Axis

Pimp-My-Axis creates virtual input devices and feeds them with results of user-provided expressions combining any number of real devices.

## Configuration

Pimp-My-Axis is configured by way of a YAML file.

### Example

```yaml
# Mapping from arbitrary device names to device matchers.
real_devices:
  # Matched by vendor and product ID.
  my_joystick:
    vendor_id: 0xdead
    product_id: 0xbeef
  # Matched by event device path.
  my_throttle: /dev/input/event8

virt_devices:
  # For now, the names of virtual devices are only used for logging.
  my_virtual_device:
    axes:
      # The following defines the "Throttle" axis on our new input device, whose values will range from -255 to 255.
      # Whenever the throttle axis of my_throttle or my_joystick changes, the expression will be reevaluated and the new axis value sent.
      Throttle:
        min: -255
        max: 255
        expr: "my_throttle:Throttle - my_joystick:Throttle"
```

Currently supported axis names are: `X`, `Y`, `Z`, `RX`, `RY`, `RZ`, `Throttle`, `Rudder`, `Wheel`, `Gas`, `Brake`.

The grammar for axis expressions can be found [here](src/expr/grammar.pest) and is pretty bare bones at the moment, but will be expanded.

### Location

By default, Pimp-My-Axis will attempt to read the first of the following files which it finds:
* `$XDG_CONFIG_HOME/pimp-my-axis/config.yml` if XDG_CONFIG_HOME is set, otherwise `~/.config/pimp-my-axis/config.yml`
* `$XDG_CONFIG_DIRS/pimp-my-axis/config.yml` if XDG_CONFIG_DIRS is set, otherwise `/etc/xdg/pimp-my-axis/config.yml`
* `/etc/pimp-my-axis/config.yml`

This can be overridden using the CLI option `--config <config>`, in which case only the given file will be tried.

## Permissions

Pimp-My-Axis needs read access to all used input event devices and read/write access to `/dev/uinput`. The easiest way to achieve this is 
to run the program as root, but other methods are possible.
