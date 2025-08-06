# hyprfocus
This is an activity/screen time tracker for hyprland.

This is achieved by using the hyprfocusd daemon to record all focus events in a log.  
The hyprfocus CLI will use these logs to generate reports on your activity.

## Usage

hyprfocus [--titles | -t | --class CLASS_NAME | -c CLASS_NAME | --idle | --resume]

## Configuration

hyprland.conf
```
exec-once = hyprfocusd
```
Add the hyprfocusd daemon to the AUTOSTART section of your hyprland config, otherwise no data will be collected and this is useless.

hypridle.conf
```
listener {
    timeout = 180
    on-timeout = hyprfocus --idle
    on-resume = hyprfocus --resume
}
```
Accurate focus data requires that hyprfocus is informed of idle events. Above is an example of how to set that up using hypridle

## Todo

- [ ] Better Usage/Help text
- [ ] Multi Day Reports
- [ ] Arbitrary Interval Reports
- [ ] Handle table formatting for long durations and extra wide characters
- [ ] Output all errors to a log file (~/.local/share/hyprfocus/hyprfocus.log)
- [ ] Implement settings and args for cutoffs for long reports
#### Configuration
- [ ] hyprfocus.conf
    - [ ] display settings
        - [ ] fancy_timeline
        - [ ] characters
        - [ ] colors
        - [ ] layout
        - [ ] minimum duration to display
        - [ ] show labels next to timelines in multi
    - [ ] default argument settings
    - [ ] ignore settings
    - [ ] category settings
