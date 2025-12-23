# hyprlog
This is an activity/screen time tracker for hyprland.

This is achieved by using the hyprlogd daemon to record all focus events in a log.  
The hyprlog CLI will use these logs to generate reports on your activity.

## Screenshots
Default

![Defaulth](./screenshots/default.png)
Multi-Line

![Multi-Line](./screenshots/multi-line.png)
Multi-Day

![Multi-Day](./screenshots/multi-day.png)

## Usage

Usage: hyprlog  
[ --help | -h ]  
[ --full | -f ]  
[ --multi | -m ]  
[ --days DAY_COUNT | -d DAY_COUNT ]  
[ --class CLASS_NAME | -c CLASS_NAME ]  
[ --idle | --resume]  

## Configuration

hyprland.conf
```
exec-once = hyprlogd
```
Add the hyprlogd daemon to the AUTOSTART section of your hyprland config, otherwise no data will be collected and this is useless.

hypridle.conf
```
listener {
    timeout = 180
    on-timeout = hyprlog --idle
    on-resume = hyprlog --resume
}
```
Accurate focus data requires that hyprlog is informed of idle events. Above is an example of how to set that up using hypridle
