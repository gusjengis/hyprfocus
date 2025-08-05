# hyprfocus
This is an activity/screen time tracker for hyprland.
This is achieved by using the hyprfocusd daemon to record all focus events in a log.
The hyprfocus CLI will use these logs to generate reports on your activity.

## Usage

todo!()

## Configuration

Add the hyprfocusd daemon to the AUTOSTART section of your hyprland config, otherwise no data will be collected and this is useless.

hyprland.conf
``` conf
exec-once = hyprfocusd
```

Accurate focus data requires that hyprfocus is informed of idle events. Here is an example of how to set that up using hypridle

hypridle.conf
``` conf
listener {
    timeout = 180
    on-timeout = hyprfocus --idle
    on-resume = hyprfocus --resume
}```
